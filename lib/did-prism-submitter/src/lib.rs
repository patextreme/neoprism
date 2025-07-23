use identus_did_prism::prelude::SignedPrismOperation;

use crate::dlt::TxId;

pub mod dlt;

#[async_trait::async_trait]
pub trait DltSink {
    async fn publish_operations(&self, operations: Vec<SignedPrismOperation>) -> Result<TxId, String>;
}
