use prism_core::{
    did::CanonicalPrismDid,
    dlt::{cardano::OuraFileSource, DltSource},
    protocol::resolver::resolve,
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
        let block_timestamp = published_atala_object.block_timestamp;
        let signed_operations = block.map(|i| i.operations).unwrap_or_default();
        for (idx, signed_operation) in signed_operations.into_iter().enumerate() {
            let _ = store
                .insert(
                    signed_operation,
                    block_timestamp.clone().into_operation_ts(idx),
                )
                .await;
        }
    }

    // test resolution
    let did = CanonicalPrismDid::from_suffix_str(
        "70f163eb8ec772ee53a25de55a5e0b4c04f346406b459427a469a9b8509e3ec4",
    )
    .unwrap();
    let ops = store.get_by_did(&did).await.unwrap().unwrap();
    let did_data = resolve(ops);
    println!("{:?}", did_data);
}
