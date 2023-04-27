use std::rc::Rc;

use crate::proto::{
    CreateDidOperation, DeactivateDidOperation, ProtocolVersionUpdateOperation, UpdateDidOperation,
};

use super::{DidStateInternalMap, OperationProcessor, OperationProcessorLike, ProcessError};

pub struct V1Processor;

impl OperationProcessorLike for V1Processor {
    fn create_did(
        &self,
        state: Rc<DidStateInternalMap>,
        operation: CreateDidOperation,
    ) -> Result<super::DidStateInternalMap, super::ProcessError> {
        todo!()
    }

    fn update_did(
        &self,
        state: Rc<DidStateInternalMap>,
        operation: UpdateDidOperation,
    ) -> Result<super::DidStateInternalMap, super::ProcessError> {
        todo!()
    }

    fn deactivate_did(
        &self,
        state: Rc<DidStateInternalMap>,
        operation: DeactivateDidOperation,
    ) -> Result<super::DidStateInternalMap, super::ProcessError> {
        todo!()
    }

    fn protocol_version_update(
        &self,
        operation: ProtocolVersionUpdateOperation,
    ) -> Result<OperationProcessor, ProcessError> {
        todo!()
    }
}
