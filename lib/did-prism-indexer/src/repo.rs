use identus_did_prism::did::CanonicalPrismDid;
use identus_did_prism::dlt::{DltCursor, OperationMetadata};
use identus_did_prism::proto::SignedPrismOperation;
use identus_did_prism::utils::paging::Paginated;

#[async_trait::async_trait]
pub trait OperationRepo {
    type Error: std::error::Error;

    async fn get_all_dids(&self, page: u32, page_size: u32) -> Result<Paginated<CanonicalPrismDid>, Self::Error>;

    async fn get_operations_by_did(
        &self,
        did: &CanonicalPrismDid,
    ) -> Result<Vec<(OperationMetadata, SignedPrismOperation)>, Self::Error>;

    async fn insert_operations(
        &self,
        operations: Vec<(OperationMetadata, SignedPrismOperation)>,
    ) -> Result<(), Self::Error>;

    async fn insert_operation(
        &self,
        signed_operation: SignedPrismOperation,
        metadata: OperationMetadata,
    ) -> Result<(), Self::Error> {
        self.insert_operations(vec![(metadata, signed_operation)]).await
    }
}

#[async_trait::async_trait]
pub trait DltCursorRepo {
    type Error: std::error::Error;

    async fn set_cursor(&self, cursor: DltCursor) -> Result<(), Self::Error>;
    async fn get_cursor(&self) -> Result<Option<DltCursor>, Self::Error>;
}
