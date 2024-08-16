use prism_core::dlt::{DltSource, OperationMetadata};
use prism_core::store::OperationStore;
use prism_storage::PostgresDb;

pub struct DltSyncWorker<Src> {
    store: PostgresDb,
    source: Src,
}

impl<Src> DltSyncWorker<Src>
where
    Src: DltSource,
{
    pub fn new(store: PostgresDb, source: Src) -> Self {
        Self { store, source }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let mut rx = self.source.receiver().expect("Unable to create a DLT source");

        while let Some(published_atala_object) = rx.recv().await {
            let block = published_atala_object.atala_object.block_content;
            let block_metadata = published_atala_object.block_metadata;
            let signed_operations = block.map(|i| i.operations).unwrap_or_default();
            let tx = self.store.begin().await?;
            for (idx, signed_operation) in signed_operations.into_iter().enumerate() {
                if signed_operation
                    .operation
                    .as_ref()
                    .and_then(|i| i.operation.as_ref())
                    .is_none()
                {
                    continue;
                }

                let insert_result = tx
                    .insert_operation(
                        signed_operation,
                        OperationMetadata {
                            block_metadata: block_metadata.clone(),
                            osn: idx as u32,
                        },
                    )
                    .await;

                if let Err(e) = insert_result {
                    log::error!("{:?}", e);
                }
            }
            tx.commit().await?;
        }
        Ok(())
    }
}
