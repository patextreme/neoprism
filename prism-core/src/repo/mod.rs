use crate::did::CanonicalPrismDid;
use crate::dlt::{DltCursor, OperationMetadata};
use crate::proto::SignedAtalaOperation;
use crate::utils::paging::Paginated;

#[async_trait::async_trait]
pub trait OperationRepo {
    type Error: std::error::Error;

    async fn get_operations_by_did(
        &self,
        did: &CanonicalPrismDid,
    ) -> Result<Vec<(OperationMetadata, SignedAtalaOperation)>, Self::Error>;

    async fn insert_operation(
        &self,
        signed_operation: SignedAtalaOperation,
        metadata: OperationMetadata,
    ) -> Result<(), Self::Error>;

    async fn get_all_dids(&self, page: u64, page_size: u64) -> Result<Paginated<CanonicalPrismDid>, Self::Error>;
}

#[async_trait::async_trait]
pub trait DltCursorRepo {
    type Error: std::error::Error;

    async fn set_cursor(&self, cursor: DltCursor) -> Result<(), Self::Error>;
    async fn get_cursor(&self) -> Result<Option<DltCursor>, Self::Error>;
}
