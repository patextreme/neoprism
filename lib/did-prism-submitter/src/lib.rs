use identus_did_prism::prelude::SignedPrismOperation;

pub mod dlt;

#[async_trait::async_trait]
pub trait DltSink {
    async fn publish(&self, ops: Vec<SignedPrismOperation>) -> Result<(), String>;
}
