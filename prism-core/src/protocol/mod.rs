use self::v1::V1Processor;
use crate::{
    crypto::hash::Sha256Digest,
    did::{
        self,
        operation::{
            CreateOperationParsingError, DeactivateOperationParsingError, PublicKey, PublicKeyId,
            Service, ServiceEndpoint, ServiceId, ServiceType, UpdateOperationParsingError,
        },
        CanonicalPrismDid, DidState,
    },
    dlt::OperationMetadata,
    proto::{
        atala_operation::Operation, CreateDidOperation, DeactivateDidOperation,
        ProtocolVersionUpdateOperation, SignedAtalaOperation, UpdateDidOperation,
    },
};
use enum_dispatch::enum_dispatch;
use std::rc::Rc;

pub mod resolver;
mod v1;

// TODO: restore test
// #[cfg(test)]
// #[path = "protocol_tests.rs"]
// mod tests;

#[derive(Debug, Clone)]
pub struct ProtocolParameter {
    pub max_services: usize,
    pub max_public_keys: usize,
    pub max_id_size: usize,
    pub max_type_size: usize,
    pub max_service_endpoint_size: usize,
}

impl Default for ProtocolParameter {
    fn default() -> Self {
        Self {
            max_services: 50,
            max_public_keys: 50,
            max_id_size: 50,
            max_type_size: 100,
            max_service_endpoint_size: 300,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("Unable to derive Did from operation")]
    DidConversionError(#[from] did::DidParsingError),
    #[error("Unable to encode operation to bytes")]
    EncodeError(#[from] prost::EncodeError),
    #[error("Operation is empty")]
    EmptyOperation,
    #[error("Unexpected operation type: {0}")]
    UnexpectedOperationType(String),
    #[error("The conflict with the exisint DID state: {0}")]
    DidStateConflict(String),
    #[error("Create operation cannot be parsed: {0}")]
    CreateOperationParseError(#[from] CreateOperationParsingError),
    #[error("Update operation cannot be parsed: {0}")]
    UpdateOperationParseError(#[from] UpdateOperationParsingError),
    #[error("Deactivate operation cannot be parsed: {0}")]
    DeactivateOperationParseError(#[from] DeactivateOperationParsingError),
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
}

#[derive(Debug, Clone)]
struct Revocable<T> {
    inner: T,
    added_at: OperationMetadata,
    revoked_at: Option<OperationMetadata>,
}

impl<T> Revocable<T> {
    fn new(item: T, added_at: &OperationMetadata) -> Self {
        Self {
            inner: item,
            added_at: added_at.clone(),
            revoked_at: None,
        }
    }

    fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }

    fn revoke(&mut self, revoked_at: &OperationMetadata) {
        self.revoked_at = Some(revoked_at.clone());
    }

    fn into_item(self) -> T {
        self.inner
    }

    fn get(&self) -> &T {
        &self.inner
    }

    fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

type InternalMap<K, V> = im_rc::HashMap<K, Revocable<V>>;

/// A struct optimized for mutating DID state as part of processing an operation.
#[derive(Debug, Clone)]
struct DidStateMut {
    did: Rc<CanonicalPrismDid>,
    context: Rc<Vec<String>>,
    last_operation_hash: Rc<Sha256Digest>,
    public_keys: InternalMap<PublicKeyId, PublicKey>,
    services: InternalMap<ServiceId, Service>,
}

impl DidStateMut {
    fn new(did: CanonicalPrismDid) -> Self {
        let last_operation_hash = did.suffix.clone();
        Self {
            did: Rc::new(did),
            last_operation_hash: Rc::new(last_operation_hash),
            context: Default::default(),
            public_keys: Default::default(),
            services: Default::default(),
        }
    }

    fn with_context(&mut self, context: Vec<String>) {
        self.context = context.into();
    }

    fn with_last_operation_hash(&mut self, last_operation_hash: Sha256Digest) {
        self.last_operation_hash = Rc::new(last_operation_hash)
    }

    fn add_public_key(
        &mut self,
        public_key: PublicKey,
        added_at: &OperationMetadata,
    ) -> Result<(), String> {
        if self.public_keys.contains_key(&public_key.id) {
            Err(format!(
                "Public key with id {} already exists",
                public_key.id
            ))?
        }

        let updated_map = self
            .public_keys
            .update(public_key.id.clone(), Revocable::new(public_key, added_at));
        self.public_keys = updated_map;
        Ok(())
    }

    fn revoke_public_key(
        &mut self,
        id: &PublicKeyId,
        revoke_at: &OperationMetadata,
    ) -> Result<(), String> {
        let Some(public_key) = self.public_keys.get_mut(id) else {
            Err(format!("Public key with id {:?} does not exist", id))?
        };

        if public_key.is_revoked() {
            Err(format!("Public key with id {:?} is already revoked", id))?
        }

        public_key.revoke(revoke_at);
        Ok(())
    }

    fn add_service(
        &mut self,
        service: Service,
        added_at: &OperationMetadata,
    ) -> Result<(), String> {
        if self.services.contains_key(&service.id) {
            Err(format!("Service with id {:?} already exists", service.id))?
        }

        let updated_map = self
            .services
            .update(service.id.clone(), Revocable::new(service, added_at));
        self.services = updated_map;
        Ok(())
    }

    fn revoke_service(
        &mut self,
        id: &ServiceId,
        revoke_at: &OperationMetadata,
    ) -> Result<(), String> {
        let Some(service) = self.services.get_mut(id) else {
            Err(format!("Service with id {:?} does not exist", id))?
        };

        if service.is_revoked() {
            Err(format!("Service with id {:?} is already revoked", id))?
        }

        service.revoke(revoke_at);
        Ok(())
    }

    fn update_service_type(&mut self, id: &ServiceId, new_type: ServiceType) -> Result<(), String> {
        let Some(service) = self.services.get_mut(id) else {
            Err(format!("Service with id {:?} does not exist", id))?
        };

        if service.is_revoked() {
            Err(format!("Service with id {:?} is revoked", id))?
        }

        service.get_mut().r#type = new_type;
        Ok(())
    }

    fn update_service_endpoint(
        &mut self,
        id: &ServiceId,
        new_endpoint: ServiceEndpoint,
    ) -> Result<(), String> {
        let Some(service) = self.services.get_mut(id) else {
            Err(format!("Service with id {:?} does not exist", id))?
        };

        if service.is_revoked() {
            Err(format!("Service with id {:?} is revoked", id))?
        }

        service.get_mut().service_endpoints = new_endpoint;
        Ok(())
    }

    fn finalize(self) -> DidState {
        let did: CanonicalPrismDid = (*self.did).clone();
        let context: Vec<String> = self
            .context
            .iter()
            .map(|s| s.as_str().to_string())
            .collect();
        let last_operation_hash: Sha256Digest = (*self.last_operation_hash).clone();
        let public_keys: Vec<PublicKey> = self
            .public_keys
            .into_iter()
            .filter(|(_, i)| !i.is_revoked())
            .map(|(_, i)| i.into_item())
            .collect();
        let services: Vec<Service> = self
            .services
            .into_iter()
            .filter(|(_, i)| !i.is_revoked())
            .map(|(_, i)| i.into_item())
            .collect();
        DidState {
            did,
            context,
            last_operation_hash,
            public_keys,
            services,
        }
    }
}

struct DidStateOps {
    state: DidStateMut,
    processor: OperationProcessorAny,
}

impl DidStateOps {
    fn new(
        signed_operation: SignedAtalaOperation,
        metadata: OperationMetadata,
    ) -> Result<Self, ProcessError> {
        let Some(operation) = &signed_operation.operation else {
            Err(ProcessError::EmptyOperation)?
        };

        let did = CanonicalPrismDid::from_operation(operation)?;
        match &operation.operation {
            Some(Operation::CreateDid(op)) => {
                let initial_state = DidStateMut::new(did);
                let processor = OperationProcessorAny::V1(V1Processor::default());
                let candidate_state = processor.create_did(&initial_state, op.clone(), metadata)?;
                processor.check_signature(&candidate_state, &signed_operation)?;
                Ok(Self {
                    state: candidate_state,
                    processor,
                })
            }
            Some(_) => Err(ProcessError::UnexpectedOperationType(
                "Operation type must be CreateDid".to_string(),
            )),
            None => Err(ProcessError::EmptyOperation),
        }
    }

    fn process(self, signed_operation: SignedAtalaOperation, metadata: OperationMetadata) -> Self {
        let signature_verification = self
            .processor
            .check_signature(&self.state, &signed_operation);
        if signature_verification.is_err() {
            return self;
        }

        let Some(operation) = signed_operation.operation else {
            return self;
        };

        let process_result = match operation.operation {
            Some(Operation::CreateDid(_)) => Err(ProcessError::UnexpectedOperationType(
                "Operation type cannot be CreateDid".to_string(),
            )),
            Some(Operation::UpdateDid(op)) => self
                .processor
                .update_did(&self.state, op, metadata)
                .map(|s| (Some(s), None)),
            Some(Operation::DeactivateDid(op)) => self
                .processor
                .deactivate_did(&self.state, op, metadata)
                .map(|s| (Some(s), None)),
            Some(Operation::ProtocolVersionUpdate(op)) => self
                .processor
                .protocol_version_update(op, metadata)
                .map(|s| (None, Some(s))),
            None => Err(ProcessError::EmptyOperation),
        };

        match process_result {
            Ok((Some(state), None)) => Self { state, ..self },
            Ok((None, Some(processor))) => Self { processor, ..self },
            Ok((Some(state), Some(processor))) => Self { state, processor },
            _ => self,
        }
    }

    fn finalize(self) -> DidState {
        self.state.finalize()
    }
}

#[enum_dispatch]
trait OperationProcessor {
    fn check_signature(
        &self,
        state: &DidStateMut,
        signed_operation: &SignedAtalaOperation,
    ) -> Result<(), ProcessError>;

    fn create_did(
        &self,
        state: &DidStateMut,
        operation: CreateDidOperation,
        metadata: OperationMetadata,
    ) -> Result<DidStateMut, ProcessError>;

    fn update_did(
        &self,
        state: &DidStateMut,
        operation: UpdateDidOperation,
        metadata: OperationMetadata,
    ) -> Result<DidStateMut, ProcessError>;

    fn deactivate_did(
        &self,
        state: &DidStateMut,
        operation: DeactivateDidOperation,
        metadata: OperationMetadata,
    ) -> Result<DidStateMut, ProcessError>;

    fn protocol_version_update(
        &self,
        operation: ProtocolVersionUpdateOperation,
        metadata: OperationMetadata,
    ) -> Result<OperationProcessorAny, ProcessError>;
}

#[enum_dispatch(OperationProcessor)]
#[derive(Debug, Clone)]
enum OperationProcessorAny {
    V1(V1Processor),
}
