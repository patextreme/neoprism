use crate::did::CanonicalPrismDid;
use crate::dlt::OperationMetadata;
use crate::proto::SignedAtalaOperation;

#[derive(Debug, Clone)]
pub struct DltCursor {
    pub slot: u64,
    pub block_hash: Vec<u8>,
}

#[async_trait::async_trait]
pub trait OperationStore {
    type Error: std::error::Error;

    async fn get_operations_by_did(
        &self,
        did: &CanonicalPrismDid,
    ) -> Result<Vec<(OperationMetadata, SignedAtalaOperation)>, Self::Error>;

    async fn insert(
        &self,
        signed_operation: SignedAtalaOperation,
        metadata: OperationMetadata,
    ) -> Result<(), Self::Error>;
}

#[async_trait::async_trait]
pub trait DltCursorStore {
    type Error: std::error::Error;

    async fn set_cursor(&self, cursor: DltCursor) -> Result<(), Self::Error>;
    async fn get_cursor(&self) -> Result<Option<DltCursor>, Self::Error>;
}
