use crate::{
    crypto::hash::Sha256Digest,
    did::{self, CanonicalPrismDid},
    proto::{
        atala_operation::Operation, AtalaOperation, CreateDidOperation, DeactivateDidOperation,
        ProtocolVersionUpdateOperation, PublicKey, Service, UpdateDidOperation,
    },
};
use std::rc::Rc;

use self::v1::V1Processor;

mod v1;

#[derive(Debug, Clone, thiserror::Error)]
pub enum ProcessError {
    #[error("unable to derive Did from operation")]
    DidConversionError(#[from] did::ParseError),
    #[error("operation is empty")]
    EmptyOperation,
}

struct DidStateInternalMap {
    did: CanonicalPrismDid,
    last_operation_hash: Sha256Digest,
    public_keys: Vec<Rc<PublicKey>>,
    services: Vec<Rc<Service>>,
    is_deactivated: bool,
}

struct DidStateOps {
    internal_map: Rc<DidStateInternalMap>,
    processor: Rc<OperationProcessor>,
}

impl DidStateOps {
    fn new(operation: AtalaOperation) -> Result<Self, ProcessError> {
        let did = CanonicalPrismDid::from_operation(operation)?;
        let last_operation_hash = did.suffix.clone();
        let internal_map = DidStateInternalMap {
            did,
            last_operation_hash,
            public_keys: Default::default(),
            services: Default::default(),
            is_deactivated: false,
        };
        Ok(Self {
            internal_map: Rc::new(internal_map),
            processor: Rc::new(OperationProcessor::V1(V1Processor)),
        })
    }

    fn process(self, operation: AtalaOperation) -> Self {
        let orig_state = self.internal_map.clone();
        let orig_processor = self.processor.clone();
        let process_result =
            match operation.operation {
                Some(Operation::CreateDid(op)) => self
                    .processor
                    .create_did(self.internal_map, op)
                    .map(|s| Self {
                        internal_map: Rc::new(s),
                        ..self
                    }),
                Some(Operation::UpdateDid(op)) => self
                    .processor
                    .update_did(self.internal_map, op)
                    .map(|s| Self {
                        internal_map: Rc::new(s),
                        ..self
                    }),
                Some(Operation::DeactivateDid(op)) => self
                    .processor
                    .deactivate_did(self.internal_map, op)
                    .map(|s| Self {
                        internal_map: Rc::new(s),
                        ..self
                    }),
                Some(Operation::ProtocolVersionUpdate(op)) => {
                    self.processor.protocol_version_update(op).map(|p| Self {
                        processor: Rc::new(p),
                        ..self
                    })
                }
                None => Err(ProcessError::EmptyOperation),
            };

        process_result.unwrap_or_else(|e| {
            log::error!("Error processing operation: {}", e);
            Self {
                internal_map: orig_state,
                processor: orig_processor,
            }
        })
    }
}

trait OperationProcessorLike {
    fn create_did(
        &self,
        state: Rc<DidStateInternalMap>,
        operation: CreateDidOperation,
    ) -> Result<DidStateInternalMap, ProcessError>;

    fn update_did(
        &self,
        state: Rc<DidStateInternalMap>,
        operation: UpdateDidOperation,
    ) -> Result<DidStateInternalMap, ProcessError>;

    fn deactivate_did(
        &self,
        state: Rc<DidStateInternalMap>,
        operation: DeactivateDidOperation,
    ) -> Result<DidStateInternalMap, ProcessError>;

    fn protocol_version_update(
        &self,
        operation: ProtocolVersionUpdateOperation,
    ) -> Result<OperationProcessor, ProcessError>;
}

enum OperationProcessor {
    V1(V1Processor),
}

impl OperationProcessorLike for OperationProcessor {
    fn create_did(
        &self,
        state: Rc<DidStateInternalMap>,
        operation: CreateDidOperation,
    ) -> Result<DidStateInternalMap, ProcessError> {
        match self {
            Self::V1(p) => p.create_did(state, operation),
        }
    }

    fn update_did(
        &self,
        state: Rc<DidStateInternalMap>,
        operation: UpdateDidOperation,
    ) -> Result<DidStateInternalMap, ProcessError> {
        match self {
            Self::V1(p) => p.update_did(state, operation),
        }
    }

    fn deactivate_did(
        &self,
        state: Rc<DidStateInternalMap>,
        operation: DeactivateDidOperation,
    ) -> Result<DidStateInternalMap, ProcessError> {
        match self {
            Self::V1(p) => p.deactivate_did(state, operation),
        }
    }

    fn protocol_version_update(
        &self,
        operation: ProtocolVersionUpdateOperation,
    ) -> Result<OperationProcessor, ProcessError> {
        match self {
            Self::V1(p) => p.protocol_version_update(operation).map(|p| p.into()),
        }
    }
}
