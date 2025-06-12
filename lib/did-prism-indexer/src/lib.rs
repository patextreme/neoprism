#![feature(error_reporter)]

use identus_apollo::hash::sha256;
use identus_apollo::hex::HexStr;
use identus_did_prism::did::CanonicalPrismDid;
use identus_did_prism::dlt::{OperationMetadata, PublishedPrismObject};
use identus_did_prism::proto::prism_operation::Operation;
use identus_did_prism::proto::{PrismOperation, SignedPrismOperation};
use prost::Message;
use tokio::sync::mpsc::Receiver;

use crate::repo::{IndexedOperation, OperationRepo, RawOperationId};

pub mod dlt;
pub mod repo;

pub trait DltSource {
    fn receiver(self) -> Result<Receiver<PublishedPrismObject>, String>;
}

/// Run indexer loop until no more operation to index
pub async fn run_indexer_loop<Repo>(repo: &Repo) -> anyhow::Result<()>
where
    Repo: OperationRepo + Send,
    <Repo as OperationRepo>::Error: Send + Sync + 'static,
{
    loop {
        let unindexed_operations = repo.get_unindexed_raw_operations().await?;
        let mut indexed_operations = Vec::with_capacity(unindexed_operations.len());

        if unindexed_operations.is_empty() {
            return Ok(());
        }

        tracing::info!("Indexing {} operations", unindexed_operations.len());
        for (id, meta, signed_operation) in unindexed_operations {
            let index_op = index_from_signed_operation(id, signed_operation)
                .inspect_err(|e| {
                    tracing::warn!(
                        ?meta,
                        "SignedPrismOperation on {:?} is ignored since it cannot be indexed. ()",
                        e
                    )
                })
                .unwrap_or(IndexedOperation::Ignored { raw_operation_id: id });
            indexed_operations.push(index_op);
        }

        repo.insert_indexed_operations(indexed_operations).await?;
    }
}

/// Run sync loop until DLT source is closed
pub async fn run_sync_loop<Repo, Src>(repo: &Repo, source: Src) -> anyhow::Result<()>
where
    Src: DltSource,
    Repo: OperationRepo + Send,
    <Repo as OperationRepo>::Error: Send + Sync + 'static,
{
    let mut rx = source.receiver().expect("Unable to create a DLT source");

    while let Some(published_prism_object) = rx.recv().await {
        let block = published_prism_object.prism_object.block_content;
        let block_metadata = published_prism_object.block_metadata;
        let signed_operations = block.map(|i| i.operations).unwrap_or_default();

        let mut insert_batch = Vec::with_capacity(signed_operations.len());
        for (idx, signed_operation) in signed_operations.into_iter().enumerate() {
            let has_operation = signed_operation
                .operation
                .as_ref()
                .and_then(|i| i.operation.as_ref())
                .is_some();

            if !has_operation {
                continue;
            }

            insert_batch.push((
                OperationMetadata {
                    block_metadata: block_metadata.clone(),
                    osn: idx as u32,
                },
                signed_operation,
            ));
        }

        let insert_result = repo.insert_raw_operations(insert_batch).await;
        if let Err(e) = insert_result {
            tracing::error!("{:?}", e);
        }
    }
    Ok(())
}

fn index_from_signed_operation(
    raw_operation_id: RawOperationId,
    signed_operation: SignedPrismOperation,
) -> anyhow::Result<IndexedOperation> {
    match signed_operation.operation {
        Some(operation) => index_from_operation(raw_operation_id, operation),
        None => Err(anyhow::anyhow!("operation does not exist in PrismOperation")),
    }
}

fn index_from_operation(
    raw_operation_id: RawOperationId,
    prism_operation: PrismOperation,
) -> anyhow::Result<IndexedOperation> {
    let operation_hash = sha256(prism_operation.encode_to_vec());
    match prism_operation.operation {
        Some(Operation::CreateDid(_)) => Ok(IndexedOperation::Ssi {
            raw_operation_id,
            did: CanonicalPrismDid::from_operation(&prism_operation)?,
        }),
        Some(Operation::UpdateDid(op)) => Ok(IndexedOperation::Ssi {
            raw_operation_id,
            did: CanonicalPrismDid::from_suffix_str(&op.id)?,
        }),
        Some(Operation::DeactivateDid(op)) => Ok(IndexedOperation::Ssi {
            raw_operation_id,
            did: CanonicalPrismDid::from_suffix_str(&op.id)?,
        }),
        Some(Operation::ProtocolVersionUpdate(op)) => Ok(IndexedOperation::Ssi {
            raw_operation_id,
            did: CanonicalPrismDid::from_suffix_str(&op.proposer_did)?,
        }),
        Some(Operation::CreateStorageEntry(op)) => Ok(IndexedOperation::Vdr {
            raw_operation_id,
            operation_hash: operation_hash.to_vec(),
            prev_operation_hash: None,
            did: Some(CanonicalPrismDid::from_suffix(HexStr::from(
                op.did_prism_hash.as_slice(),
            ))?),
        }),
        Some(Operation::UpdateStorageEntry(op)) => Ok(IndexedOperation::Vdr {
            raw_operation_id,
            operation_hash: operation_hash.to_vec(),
            prev_operation_hash: Some(op.previous_operation_hash),
            did: None,
        }),
        Some(Operation::DeactivateStorageEntry(op)) => Ok(IndexedOperation::Vdr {
            raw_operation_id,
            operation_hash: operation_hash.to_vec(),
            prev_operation_hash: Some(op.previous_operation_hash),
            did: None,
        }),
        None => Err(anyhow::anyhow!("operation does not exist in PrismOperation")),
    }
}
