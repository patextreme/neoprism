use crate::{
    did::{self, CanonicalPrismDid},
    dlt::OperationMetadata,
    prelude::StdError,
    proto::{atala_operation::Operation, AtalaOperation, SignedAtalaOperation},
};
use std::collections::HashMap;

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
    StorageBackendError(Box<dyn std::error::Error>),
    #[error("Storage encoding/decoding error: {0}")]
    StorageEncodingError(Box<dyn std::error::Error>),
}

#[async_trait::async_trait]
pub trait OperationStore {
    async fn get_by_did(
        &mut self,
        did: &CanonicalPrismDid,
    ) -> Result<Vec<(OperationMetadata, SignedAtalaOperation)>, OperationStoreError>;

    async fn insert(
        &mut self,
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
    async fn set_cursor(&mut self, cursor: DltCursor) -> Result<(), CursorStoreError>;
    async fn get_cursor(&mut self) -> Result<Option<DltCursor>, CursorStoreError>;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryStore {
    operations: HashMap<CanonicalPrismDid, Vec<(OperationMetadata, SignedAtalaOperation)>>,
    cursor: Option<DltCursor>,
}

#[async_trait::async_trait]
impl OperationStore for InMemoryStore {
    async fn insert(
        &mut self,
        signed_operation: SignedAtalaOperation,
        metadata: OperationMetadata,
    ) -> Result<(), OperationStoreError> {
        let did = get_did_from_signed_operation(&signed_operation)?;
        let did_str = did.to_string();
        self.operations
            .entry(did)
            .or_insert_with(Vec::new)
            .push((metadata, signed_operation));

        let did_count = self.operations.len();
        let ops_count = self.operations.values().map(|v| v.len()).sum::<usize>();
        log::debug!(
            "Inserted operation of {}. Store contains {} DIDs and {} operations.",
            did_str,
            did_count,
            ops_count
        );

        Ok(())
    }

    async fn get_by_did(
        &mut self,
        did: &CanonicalPrismDid,
    ) -> Result<Vec<(OperationMetadata, SignedAtalaOperation)>, OperationStoreError> {
        let result = self.operations.get(did).cloned();

        log::info!(
            "Read operation successfully. Got {} operations",
            result.as_ref().map(|i| i.len()).unwrap_or_default()
        );

        Ok(result.unwrap_or_default())
    }
}

#[async_trait::async_trait]
impl DltCursorStore for InMemoryStore {
    async fn set_cursor(&mut self, cursor: DltCursor) -> Result<(), CursorStoreError> {
        self.cursor = Some(cursor);
        Ok(())
    }

    async fn get_cursor(&mut self) -> Result<Option<DltCursor>, CursorStoreError> {
        Ok(self.cursor.clone())
    }
}
