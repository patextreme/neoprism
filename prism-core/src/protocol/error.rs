use crate::did::error::{Error as DidError, PublicKeyIdError};
use crate::did::operation::{KeyUsage, PublicKeyId, ServiceId};

#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum ProcessError {
    #[from]
    #[display("{source}")]
    DidOperationInvalid { source: DidError },
    #[display("did state initialization requires operation to be CreateOperation")]
    DidStateInitFromNonCreateOperation,
    #[display("did state update cannot be performed by CreateOperation")]
    DidStateUpdateFromCreateOperation,
    #[display("operation is missing from SignedAtalaOperation")]
    SignedAtalaOperationMissingOperation,
    #[display("invalid signed_with key id in SignedAtalaOperation")]
    SignedAtalaOperationInvalidSignedWith { source: PublicKeyIdError },
    #[display("signed_with key id {id} not found")]
    SignedAtalaOperationSignedWithKeyNotFound { id: PublicKeyId },
    #[display("signed_with key id {id} has usage of {usage:?} which is not a master key")]
    SignedAtalaOperationSignedWithNonMasterKey { id: PublicKeyId, usage: KeyUsage },
    #[display("signature verification failed for SignedAtalaOperation")]
    SignedAtalaOperationInvalidSignature,
    #[from]
    #[display("applied operation has conflict with the current did state")]
    DidStateConflict { source: DidStateConflictError },
}

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum DidStateConflictError {
    #[display("applied operation does not have matching previous_operation_hash in the current did state")]
    UnmatchedPreviousOperationHash,
    #[display("cannot add public key since key id {id} already exist in the did state")]
    AddPublicKeyWithExistingId { id: PublicKeyId },
    #[display("cannot revoke public key since key id {id} does not exist in the did state")]
    RevokePublicKeyNotExists { id: PublicKeyId },
    #[display("cannot revoke public key since key id {id} is already revoked")]
    RevokePublicKeyIsAlreadyRevoked { id: PublicKeyId },
    #[display("cannot add service since service with id {id} already exist in the did state")]
    AddServiceWithExistingId { id: ServiceId },
    #[display("cannot revoke service since service with id {id} does not exist in the did state")]
    RevokeServiceNotExists { id: ServiceId },
    #[display("cannot revoke service since service with id {id} is already revoked")]
    RevokeServiceIsAlreadyRevoked { id: ServiceId },
    #[display("cannot update service since service with id {id} does not exist in the did state")]
    UpdateServiceNotExists { id: ServiceId },
    #[display("cannot update service since service with id {id} is already revoked")]
    UpdateServiceIsRevoked { id: ServiceId },
    #[display("did state must have at least one master must exist after updated")]
    AfterUpdateMissingMasterKey,
    #[display("did state have {actual} public keys which is greater than the limit {limit}")]
    AfterUpdatePublicKeyExceedLimit { limit: usize, actual: usize },
    #[display("did state have {actual} services which is greater than the limit {limit}")]
    AfterUpdateServiceExceedLimit { limit: usize, actual: usize },
}
