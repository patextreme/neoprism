use crate::{
    did::{self, CanonicalPrismDid},
    dlt::DltTimestamp,
    proto::{atala_operation::Operation, AtalaOperation},
};
use std::collections::HashMap;

#[derive(Debug, Clone, thiserror::Error)]
pub enum OperationStoreError {
    #[error("Unable to parse Did from operation: {0}")]
    DidParseError(#[from] did::ParseError),
    #[error("Operation is empty")]
    EmptyOperation,
}

pub enum OperationStore {
    InMemory(InMemoryOperationStore),
}

impl OperationStore {
    pub fn in_memory() -> Self {
        Self::InMemory(InMemoryOperationStore::new())
    }

    pub async fn insert(
        &mut self,
        operation: AtalaOperation,
        timestamp: DltTimestamp,
    ) -> Result<(), OperationStoreError> {
        let did = extract_insert_key(&operation)?;
        match self {
            Self::InMemory(store) => store.insert(did, operation, timestamp),
        }
    }

    pub async fn get_by_did(
        &mut self,
        did: &CanonicalPrismDid,
    ) -> Result<Option<Vec<(DltTimestamp, AtalaOperation)>>, OperationStoreError> {
        match self {
            Self::InMemory(store) => store.get_by_did(did),
        }
    }
}

fn extract_insert_key(
    atala_operation: &AtalaOperation,
) -> Result<CanonicalPrismDid, OperationStoreError> {
    match &atala_operation.operation {
        Some(Operation::CreateDid(_)) => {
            Ok(CanonicalPrismDid::from_operation(atala_operation.clone())?)
        }
        Some(Operation::UpdateDid(op)) => Ok(CanonicalPrismDid::from_suffix_str(&op.id)?),
        Some(Operation::DeactivateDid(op)) => Ok(CanonicalPrismDid::from_suffix_str(&op.id)?),
        Some(Operation::ProtocolVersionUpdate(_)) => todo!("add support for protocol update"),
        None => Err(OperationStoreError::EmptyOperation),
    }
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryOperationStore {
    operations: HashMap<CanonicalPrismDid, Vec<(DltTimestamp, AtalaOperation)>>,
}

impl InMemoryOperationStore {
    pub fn new() -> Self {
        Default::default()
    }

    fn insert(
        &mut self,
        did: CanonicalPrismDid,
        operation: AtalaOperation,
        timestamp: DltTimestamp,
    ) -> Result<(), OperationStoreError> {
        self.operations
            .entry(did)
            .or_insert_with(Vec::new)
            .push((timestamp, operation));
        Ok(())
    }

    fn get_by_did(
        &mut self,
        did: &CanonicalPrismDid,
    ) -> Result<Option<Vec<(DltTimestamp, AtalaOperation)>>, OperationStoreError> {
        Ok(self.operations.get(did).cloned())
    }
}
