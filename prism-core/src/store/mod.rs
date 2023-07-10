use crate::{
    did::{self, CanonicalPrismDid},
    dlt::OperationMetadata,
    prelude::StdError,
    proto::{atala_operation::Operation, AtalaOperation, SignedAtalaOperation},
};

pub fn get_did_from_operation(
    atala_operation: &AtalaOperation,
) -> Result<CanonicalPrismDid, OperationStoreError> {
    match &atala_operation.operation {
        Some(Operation::CreateDid(_)) => Ok(CanonicalPrismDid::from_operation(atala_operation)?),
        Some(Operation::UpdateDid(op)) => Ok(CanonicalPrismDid::from_suffix_str(&op.id)?),
        Some(Operation::DeactivateDid(op)) => Ok(CanonicalPrismDid::from_suffix_str(&op.id)?),
        Some(Operation::ProtocolVersionUpdate(op)) => {
            Ok(CanonicalPrismDid::from_suffix_str(&op.proposer_did)?)
        }
        None => Err(OperationStoreError::EmptyOperation),
    }
}

pub fn get_did_from_signed_operation(
    signed_operation: &SignedAtalaOperation,
) -> Result<CanonicalPrismDid, OperationStoreError> {
    match &signed_operation.operation {
        Some(operation) => get_did_from_operation(operation),
        None => Err(OperationStoreError::EmptyOperation),
    }
}

#[derive(Debug, Clone)]
pub struct DltCursor {
    pub slot: u64,
    pub block_hash: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum OperationStoreError {
    #[error("Unable to parse Did from operation: {0}")]
    DidParseError(#[from] did::DidParsingError),
    #[error("Operation is empty")]
    EmptyOperation,
    #[error("Operation cannot be encoded to bytes: {0}")]
    OperationEncodeError(#[from] prost::EncodeError),
    #[error("Operation canno be decoded from bytes: {0}")]
    OperationDecodeError(#[from] prost::DecodeError),
    #[error("Storage mechanism error: {0}")]
    StorageBackendError(StdError),
    #[error("Storage encoding/decoding error: {0}")]
    StorageEncodingError(StdError),
}

#[async_trait::async_trait]
pub trait OperationStore {
    async fn get_by_did(
        &self,
        did: &CanonicalPrismDid,
    ) -> Result<Vec<(OperationMetadata, SignedAtalaOperation)>, OperationStoreError>;

    async fn insert(
        &self,
        signed_operation: SignedAtalaOperation,
        metadata: OperationMetadata,
    ) -> Result<(), OperationStoreError>;
}

#[derive(Debug, thiserror::Error)]
pub enum CursorStoreError {
    #[error("Storage mechanism error: {0}")]
    StorageBackendError(StdError),
    #[error("Storage encoding/decoding error: {0}")]
    StorageEncodingError(StdError),
}

#[async_trait::async_trait]
pub trait DltCursorStore {
    async fn set_cursor(&self, cursor: DltCursor) -> Result<(), CursorStoreError>;
    async fn get_cursor(&self) -> Result<Option<DltCursor>, CursorStoreError>;
}
