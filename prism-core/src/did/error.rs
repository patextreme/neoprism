use super::operation::PublicKeyId;
use super::CanonicalPrismDid;
use crate::error::InvalidInputSizeError;
use crate::utils::codec::HexStr;

#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum Error {
    #[display("operation type provided when creating a long-form DID is not CreateDidOperation")]
    LongFormDidNotFromCreateOperation,
    #[display("operation does not exist in AtalaOperation")]
    OperationMissingFromAtalaObject,
    #[from]
    #[display("invalid did syntax")]
    InvalidDidSyntax { source: DidSyntaxError },
    #[from]
    #[display("error occurred in CreateOperation")]
    CreateOperation { source: CreateOperationError },
}

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum DidSyntaxError {
    #[display("did suffix {suffix} has invalid length")]
    DidSuffixInvalidHex {
        source: InvalidInputSizeError,
        suffix: HexStr,
    },
    #[display("did suffix {suffix} is not valid")]
    DidSuffixInvalidStr {
        source: crate::utils::codec::Error,
        suffix: String,
    },
    #[display("did encoded state {encoded_state} is not valid")]
    DidEncodedStateInvalidStr {
        source: crate::utils::codec::Error,
        encoded_state: String,
    },
    #[display("did suffix {did} cannot be decoded into protobuf message")]
    DidEncodedStateInvalidProto { source: prost::DecodeError, did: String },
    #[display("unrecognized did pattern {did}")]
    DidSyntaxInvalid {
        #[error(not(source))]
        did: String,
    },
    #[display("encoded state and did suffix do not match for {did} (expected {expected_did})")]
    DidSuffixEncodedStateUnmatched {
        did: String,
        expected_did: CanonicalPrismDid,
    },
}

#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum CreateOperationError {
    #[display("missing did_data in CreateOperation")]
    MissingDidData,
    #[display("no master key found in CreateOperation")]
    MissingMasterKey,
    #[from]
    #[display("invalid public key found in CreateOperation")]
    InvalidPublicKey(PublicKeyError),
    #[from]
    #[display("invalid service found in CreateOperation")]
    InvalidService(ServiceError),
    #[display("invalid input size for public keys")]
    TooManyPublicKeys { source: InvalidInputSizeError },
    #[display("invalid input size for services")]
    TooManyServices { source: InvalidInputSizeError },
    #[display("duplicate context found in CreateOperation")]
    DuplicateContext,
}

#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum PublicKeyError {
    #[from]
    #[display("invalid public key id {id}")]
    InvalidKeyId { source: PublicKeyIdError, id: String },
    #[display("missing key data for key id {id}")]
    MissingKeyData {
        #[error(not(source))]
        id: PublicKeyId,
    },
    #[display("unknown key usage for key id {id}")]
    UnknownKeyUsage {
        #[error(not(source))]
        id: PublicKeyId,
    },
    #[display("master key id {id} does not have key type of secp256k1")]
    MasterKeyNotSecp256k1 {
        #[error(not(source))]
        id: PublicKeyId,
    },
    #[from]
    #[display("unable to parse key data to a public key for id {id}")]
    Crypto {
        source: crate::crypto::Error,
        id: PublicKeyId,
    },
}

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum PublicKeyIdError {
    #[display("public key id is empty")]
    Empty,
    #[display("public key id is not a valid uri fragment")]
    InvalidUriFragment,
    #[display("public key id is too long")]
    TooLong { source: InvalidInputSizeError },
}

#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum ServiceError {
    #[from]
    #[display("invalid service id {id}")]
    InvalidServiceId { source: ServiceIdError, id: String },
    #[from]
    #[display("invalid service type {type_name}")]
    InvalidServiceType {
        source: ServiceTypeError,
        type_name: String,
    },
    #[from]
    #[display("invalid service endpoint {endpoint}")]
    InvalidServiceEndpoint {
        source: ServiceEndpointError,
        endpoint: String,
    },
}

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum ServiceIdError {
    #[display("service id is empty")]
    Empty,
    #[display("service id is not a valid uri fragment")]
    InvalidUriFragment,
    #[display("service id is too long")]
    TooLong { source: InvalidInputSizeError },
}

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum ServiceTypeError {
    #[display("service type exceed max size of {limit}")]
    ExceedMaxSize {
        #[error(not(source))]
        limit: usize,
    },
    #[display("service type is empty")]
    Empty,
    #[display("service type does not conform to the syntax")]
    InvalidSyntax,
}

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum ServiceEndpointError {
    #[display("service endpoint exceed max size of {limit}")]
    ExceedMaxSize {
        #[error(not(source))]
        limit: usize,
    },
    #[display("service endpoint is empty")]
    Empty,
    #[display("service endpoint does not conform to the syntax")]
    InvalidSyntax,
}
