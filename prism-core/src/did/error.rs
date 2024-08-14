use crate::error::InvalidInputSizeError;
use crate::utils::codec::HexStr;

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum Error {
    #[display("operation type provided when creating a long-form DID is not CreateDidOperation")]
    LongFormDidNotFromCreateOperation,
    #[display("operation does not exist in AtalaOperation")]
    OperationMissingFromAtalaObject,
    #[display("did {_0} does not start with did:prism:")]
    DidPrefixInvalidStr(#[error(not(source))] String),
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
    #[display("unrecognized did pattern {_0}")]
    DidSyntaxInvalid(#[error(not(source))] String),
    #[display("encoded state and did suffix do not match for {_0}")]
    DidSuffixEncodedStateUnmatched(#[error(not(source))] String),
}
