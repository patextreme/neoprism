use prism_core::{
    dlt::{DltSink, DltSource, InMemoryDlt},
    proto::AtalaBlock,
};
use std::time::Duration;

#[tokio::main]
async fn main() {
    env_logger::init();

    let dlt = InMemoryDlt::new(Duration::from_secs(1));
    let (source, sink) = dlt.split();

    tokio::spawn(async move {
        let mut sink = sink;
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let atala_object = AtalaBlock { operations: vec![] };
            sink.send(atala_object);
        }
    });

    let mut rx = source.receiver();
    loop {
        let atala_object = rx.recv().await.unwrap();
        log::info!("{:?}", atala_object);
    }
}
