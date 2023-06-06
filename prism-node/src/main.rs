use prism_core::{
    did::CanonicalPrismDid,
    dlt::{
        cardano::{NetworkIdentifier, OuraN2NSource},
        DltSource,
    },
    protocol::resolver::resolve,
    store::{InMemoryOperationStore, OperationStore},
};

#[tokio::main]
async fn main() {
    env_logger::init();
    // let mut store =
    //     SurrealOperationStore::ws_root("localhost:8000", "test", "test", "root", "root")
    //         .await
    //         .unwrap();

    let mut store = InMemoryOperationStore::default();

    // let source = OuraFileSource::new("./mainnet");
    let source = OuraN2NSource::new("localhost:3001", &NetworkIdentifier::Mainnet);

    let mut rx = source.receiver().expect("unable to create a DLT source");

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
    let ops = store.get_by_did(&did).await.unwrap();
    let did_data = resolve(ops);
    println!("{:#?}", did_data);
}
