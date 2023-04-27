use prism_core::{
    dlt::{cardano::OuraFileSource, DltSource},
    proto::AtalaOperation,
    store::OperationStore,
};

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut store = OperationStore::in_memory();

    let source = OuraFileSource::new("./mainnet");
    let mut rx = source.receiver();
    while let Some(published_atala_object) = rx.recv().await {
        let block = published_atala_object.atala_object.block_content;
        let dlt_timestamp = published_atala_object.dlt_timestamp;
        let signed_operations = block.map(|i| i.operations).unwrap_or_default();
        let operations: Vec<AtalaOperation> = signed_operations
            .into_iter()
            .flat_map(|i| i.operation)
            .collect();

        for operation in operations.into_iter() {
            store
                .insert(operation, dlt_timestamp.clone())
                .await
                .unwrap();
        }
    }
}
