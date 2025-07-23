use identus_did_prism::prelude::SignedPrismOperation;

#[async_trait::async_trait]
pub trait ScheduledOperationRepo {
    type Error: std::error::Error;

    async fn get_unpublished_operations(&self) -> Result<Vec<SignedPrismOperation>, Self::Error>;
}
