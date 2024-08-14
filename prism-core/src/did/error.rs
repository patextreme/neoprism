#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid operation type: {0}")]
    InvalidOperationType(String),
    #[error("Operation does not exist")]
    OperationMissing,
    #[error("Invalid suffix length: {0}")]
    InvalidSuffixLength(String),
    #[error("Invalid suffix: {0}")]
    InvalidSuffix(#[from] crate::utils::codec::Error),
    #[error("Does not starts with 'did:prism:'")]
    InvalidPrefix,
    #[error("Unrecognized suffix format for Prism DID: {0}")]
    UnrecognizedSuffixFormat(String),
    #[error("Fail to convert encoded state to AtalaOperation")]
    InvalidEncodedState(#[from] prost::DecodeError),
    #[error("Encoded state does not match DID suffix")]
    UnmatchEncodedStateSuffix,
}
