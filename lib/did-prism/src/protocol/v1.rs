use identus_apollo::crypto::Verifiable;
use identus_apollo::hash::sha256;
use prost::Message;

use super::{
    DidStateConflictError, DidStateRc, OperationProcessor, OperationProcessorVariants, ProcessError, ProtocolParameter,
};
use crate::did::Error as DidError;
use crate::did::operation::{
    CreateOperation, DeactivateOperation, KeyUsage, PublicKeyData, PublicKeyId, UpdateOperation, UpdateOperationAction,
};
use crate::dlt::OperationMetadata;
use crate::prelude::PrismOperation;
use crate::proto::prism_operation::Operation;
use crate::proto::{
    ProtoCreateDid, ProtoDeactivateDid, ProtoProtocolVersionUpdate, ProtoUpdateDid, SignedPrismOperation,
};

#[derive(Debug, Clone)]
pub struct V1Processor {
    parameters: ProtocolParameter,
}

impl Default for V1Processor {
    fn default() -> Self {
        Self {
            parameters: ProtocolParameter::v1(),
        }
    }
}

impl OperationProcessor for V1Processor {
    fn check_signature(&self, state: &DidStateRc, signed_operation: &SignedPrismOperation) -> Result<(), ProcessError> {
        let key_id = PublicKeyId::parse(&signed_operation.signed_with, self.parameters.max_id_size)
            .map_err(|e| ProcessError::SignedPrismOperationInvalidSignedWith { source: e })?;

        let Some(pk) = state.public_keys.get(&key_id) else {
            Err(ProcessError::SignedPrismOperationSignedWithKeyNotFound { id: key_id })?
        };

        match &pk.get().data {
            PublicKeyData::Master { data } => {
                let signature = signed_operation.signature.as_slice();
                let message = signed_operation
                    .operation
                    .as_ref()
                    .ok_or(ProcessError::SignedPrismOperationMissingOperation)?
                    .encode_to_vec();

                if !data.verify(&message, signature) {
                    Err(ProcessError::SignedPrismOperationInvalidSignature)?
                }
            }
            PublicKeyData::Other { usage, .. } => Err(ProcessError::SignedPrismOperationSignedWithNonMasterKey {
                id: key_id,
                usage: *usage,
            })?,
        }

        Ok(())
    }

    fn create_did(
        &self,
        state: &DidStateRc,
        operation: ProtoCreateDid,
        metadata: OperationMetadata,
    ) -> Result<DidStateRc, ProcessError> {
        let parsed_operation = CreateOperation::parse(&self.parameters, &operation).map_err(DidError::from)?;

        // clone and mutate candidate state
        let mut candidate_state = state.clone();
        candidate_state.with_context(parsed_operation.context);
        candidate_state.with_last_operation_hash(state.did.suffix.clone());
        for pk in parsed_operation.public_keys {
            candidate_state.add_public_key(pk, &metadata)?;
        }
        for service in parsed_operation.services {
            candidate_state.add_service(service, &metadata)?;
        }

        CreateDidValidator::validate_candidate_state(&self.parameters, &candidate_state)?;
        Ok(candidate_state)
    }

    fn update_did(
        &self,
        state: &DidStateRc,
        operation: ProtoUpdateDid,
        metadata: OperationMetadata,
    ) -> Result<DidStateRc, ProcessError> {
        let parsed_operation = UpdateOperation::parse(&self.parameters, &operation).map_err(DidError::from)?;
        if parsed_operation.prev_operation_hash != *state.last_operation_hash {
            Err(DidStateConflictError::UnmatchedPreviousOperationHash)?
        }

        // clone and mutate candidate state
        let mut candidate_state = state.clone();
        let atala_operation = PrismOperation {
            operation: Some(Operation::UpdateDid(operation)),
        };
        candidate_state.with_last_operation_hash(sha256(atala_operation.encode_to_vec()));
        for action in parsed_operation.actions {
            apply_update_action(&mut candidate_state, action, &metadata)?;
        }

        UpdateDidValidator::validate_candidate_state(&self.parameters, &candidate_state)?;
        Ok(candidate_state)
    }

    fn deactivate_did(
        &self,
        state: &DidStateRc,
        operation: ProtoDeactivateDid,
        metadata: OperationMetadata,
    ) -> Result<DidStateRc, ProcessError> {
        let parsed_operation = DeactivateOperation::parse(&operation).map_err(DidError::from)?;
        if parsed_operation.prev_operation_hash != *state.last_operation_hash {
            Err(DidStateConflictError::UnmatchedPreviousOperationHash)?
        }

        // clone and mutate candidate state
        let mut candidate_state = state.clone();
        let atala_operation = PrismOperation {
            operation: Some(Operation::DeactivateDid(operation)),
        };
        candidate_state.with_last_operation_hash(sha256(atala_operation.encode_to_vec()));
        for (id, pk) in &state.public_keys {
            if !pk.is_revoked() {
                candidate_state.revoke_public_key(id, &metadata)?;
            }
        }
        for (id, s) in &state.services {
            if !s.is_revoked() {
                candidate_state.revoke_service(id, &metadata)?;
            }
        }

        DeactivateDidValidator::validate_candidate_state(&self.parameters, &candidate_state)?;
        Ok(candidate_state)
    }

    fn protocol_version_update(
        &self,
        _: ProtoProtocolVersionUpdate,
        _: OperationMetadata,
    ) -> Result<OperationProcessorVariants, ProcessError> {
        // TODO: add support for protocol version update
        tracing::warn!("Protocol version update is not yet supported");
        Ok(self.clone().into())
    }
}

trait Validator<Op> {
    fn validate_candidate_state(param: &ProtocolParameter, state: &DidStateRc) -> Result<(), ProcessError>;
}

struct CreateDidValidator;
struct UpdateDidValidator;
struct DeactivateDidValidator;

impl Validator<ProtoCreateDid> for CreateDidValidator {
    fn validate_candidate_state(_: &ProtocolParameter, _: &DidStateRc) -> Result<(), ProcessError> {
        Ok(())
    }
}

impl Validator<ProtoUpdateDid> for UpdateDidValidator {
    fn validate_candidate_state(param: &ProtocolParameter, state: &DidStateRc) -> Result<(), ProcessError> {
        // check at least one master key exists
        let contains_master_key = state
            .public_keys
            .iter()
            .any(|(_, pk)| pk.get().usage() == KeyUsage::MasterKey);
        if !contains_master_key {
            Err(DidStateConflictError::AfterUpdateMissingMasterKey)?
        }

        // check public key count does not exceed limit
        if state.public_keys.len() > param.max_public_keys {
            Err(DidStateConflictError::AfterUpdatePublicKeyExceedLimit {
                limit: param.max_public_keys,
                actual: state.public_keys.len(),
            })?
        }

        // check service count does not exeed limit
        if state.services.len() > param.max_services {
            Err(DidStateConflictError::AfterUpdateServiceExceedLimit {
                limit: param.max_services,
                actual: state.services.len(),
            })?
        }

        Ok(())
    }
}

impl Validator<ProtoDeactivateDid> for DeactivateDidValidator {
    fn validate_candidate_state(_: &ProtocolParameter, _: &DidStateRc) -> Result<(), ProcessError> {
        Ok(())
    }
}

fn apply_update_action(
    state: &mut DidStateRc,
    action: UpdateOperationAction,
    metadata: &OperationMetadata,
) -> Result<(), DidStateConflictError> {
    match action {
        UpdateOperationAction::AddKey(pk) => state.add_public_key(pk, metadata)?,
        UpdateOperationAction::RemoveKey(id) => state.revoke_public_key(&id, metadata)?,
        UpdateOperationAction::AddService(service) => state.add_service(service, metadata)?,
        UpdateOperationAction::RemoveService(id) => state.revoke_service(&id, metadata)?,
        UpdateOperationAction::UpdateService {
            id,
            r#type,
            service_endpoints,
        } => {
            if let Some(t) = r#type {
                state.update_service_type(&id, t)?;
            }
            if let Some(ep) = service_endpoints {
                state.update_service_endpoint(&id, ep)?;
            }
        }
        UpdateOperationAction::PatchContext(ctx) => {
            state.with_context(ctx);
        }
    }

    Ok(())
}
