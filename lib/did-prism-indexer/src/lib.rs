#![feature(error_reporter)]

use identus_did_prism::dlt::{OperationMetadata, PublishedPrismObject};
use tokio::sync::mpsc::Receiver;

use crate::repo::OperationRepo;

pub mod dlt;
pub mod repo;

pub trait DltSource {
    fn receiver(self) -> Result<Receiver<PublishedPrismObject>, String>;
}

pub async fn run_sync_loop<Repo, Src>(repo: Repo, source: Src) -> anyhow::Result<()>
where
    Src: DltSource,
    Repo: OperationRepo,
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
