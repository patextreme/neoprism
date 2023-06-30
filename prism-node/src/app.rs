use prism_core::{
    dlt::{DltSource, OperationMetadata},
    store::OperationStore,
};

pub struct PrismNodeApp<Store, Src>
where
    Store: OperationStore,
    Src: DltSource,
{
    store: Store,
    source: Src,
}

impl<Store, Src> PrismNodeApp<Store, Src>
where
    Store: OperationStore,
    Src: DltSource,
{
    pub fn new(store: Store, source: Src) -> Self {
        Self { store, source }
    }

    pub async fn run(mut self) {
        let mut rx = self
            .source
            .receiver()
            .expect("Unable to create a DLT source");

        while let Some(published_atala_object) = rx.recv().await {
            let block = published_atala_object.atala_object.block_content;
            let block_metadata = published_atala_object.block_metadata;
            let signed_operations = block.map(|i| i.operations).unwrap_or_default();
            for (idx, signed_operation) in signed_operations.into_iter().enumerate() {
                let _ = self
                    .store
                    .insert(
                        signed_operation,
                        // block_metadata.clone().into_operation_ts(idx as u32),
                        OperationMetadata {
                            block_metadata: block_metadata.clone(),
                            osn: idx as u32,
                        },
                    )
                    .await;
            }
        }
    }
}
