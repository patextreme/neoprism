use crate::{
    did::{self, CanonicalPrismDid, PrismDid},
    dlt::OperationTimestamp,
    proto::{atala_operation::Operation, AtalaOperation, SignedAtalaOperation},
};
use std::collections::HashMap;

#[derive(Debug, Clone, thiserror::Error)]
pub enum OperationStoreError {
    #[error("Unable to parse Did from operation: {0}")]
    DidParseError(#[from] did::DidParsingError),
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
        signed_operation: SignedAtalaOperation,
        timestamp: OperationTimestamp,
    ) -> Result<CanonicalPrismDid, OperationStoreError> {
        let Some(operation) = &signed_operation.operation else {
            Err(OperationStoreError::EmptyOperation)?
        };

        let did = extract_insert_key(operation)?;
        let op_type = extract_operation_type_name(operation);
        match self {
            Self::InMemory(store) => store.insert(did.clone(), signed_operation, timestamp)?,
        }
        log::info!(
            "Inserted {} operation for {}",
            op_type.unwrap_or("None"),
            did.to_string()
        );
        Ok(did)
    }

    pub async fn get_by_did(
        &mut self,
        did: &CanonicalPrismDid,
    ) -> Result<Option<Vec<(OperationTimestamp, SignedAtalaOperation)>>, OperationStoreError> {
        match self {
            Self::InMemory(store) => store.get_by_did(did),
        }
    }
}

fn extract_insert_key(
    atala_operation: &AtalaOperation,
) -> Result<CanonicalPrismDid, OperationStoreError> {
    match &atala_operation.operation {
        Some(Operation::CreateDid(_)) => Ok(CanonicalPrismDid::from_operation(atala_operation)?),
        Some(Operation::UpdateDid(op)) => Ok(CanonicalPrismDid::from_suffix_str(&op.id)?),
        Some(Operation::DeactivateDid(op)) => Ok(CanonicalPrismDid::from_suffix_str(&op.id)?),
        Some(Operation::ProtocolVersionUpdate(_)) => todo!("add support for protocol update"),
        None => Err(OperationStoreError::EmptyOperation),
    }
}

fn extract_operation_type_name(operation: &AtalaOperation) -> Option<&'static str> {
    operation.operation.as_ref().map(|o| match o {
        Operation::CreateDid(_) => "CreateDid",
        Operation::UpdateDid(_) => "UpdateDid",
        Operation::ProtocolVersionUpdate(_) => "ProtocolVersionUpdate",
        Operation::DeactivateDid(_) => "DeactivateDid",
    })
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryOperationStore {
    operations: HashMap<CanonicalPrismDid, Vec<(OperationTimestamp, SignedAtalaOperation)>>,
}

impl InMemoryOperationStore {
    pub fn new() -> Self {
        Default::default()
    }

    fn insert(
        &mut self,
        did: CanonicalPrismDid,
        signed_operation: SignedAtalaOperation,
        timestamp: OperationTimestamp,
    ) -> Result<(), OperationStoreError> {
        self.operations
            .entry(did)
            .or_insert_with(Vec::new)
            .push((timestamp, signed_operation));
        Ok(())
    }

    fn get_by_did(
        &mut self,
        did: &CanonicalPrismDid,
    ) -> Result<Option<Vec<(OperationTimestamp, SignedAtalaOperation)>>, OperationStoreError> {
        Ok(self.operations.get(did).cloned())
    }
}
