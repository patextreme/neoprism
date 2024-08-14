use std::str::FromStr;
use std::sync::LazyLock;

use enum_dispatch::enum_dispatch;
use prost::Message;
use regex::Regex;

use self::operation::{PublicKey, Service};
use crate::proto::atala_operation::Operation;
use crate::proto::AtalaOperation;
use crate::utils::codec::{Base64UrlStrNoPad, HexStr};
use crate::utils::hash::{sha256, Sha256Digest};

pub mod operation;

static CANONICAL_SUFFIX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([0-9a-f]{64}$)").expect("CANONICAL_SUFFIX_RE regex is invalid"));
static LONG_FORM_SUFFIX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([0-9a-f]{64}):([A-Za-z0-9_-]+$)").expect("LONG_FORM_SUFFIX_RE regex is invalid"));

#[enum_dispatch(PrismDidLike)]
#[derive(Clone, PartialEq, Eq, Hash, derive_more::Debug, derive_more::Display)]
pub enum PrismDid {
    #[display("{_0}")]
    #[debug("{_0}")]
    Canonical(CanonicalPrismDid),
    #[display("{_0}")]
    #[debug("{_0}")]
    LongForm(LongFormPrismDid),
}

#[derive(Clone, PartialEq, Eq, Hash, derive_more::Debug, derive_more::Display)]
#[display("did:{}:{}", self.method(), self.suffix_hex())]
#[debug("did:{}:{}", self.method(), self.suffix_hex())]
pub struct CanonicalPrismDid {
    pub suffix: Sha256Digest,
}

#[derive(Clone, PartialEq, Eq, Hash, derive_more::Debug, derive_more::Display)]
#[display("did:{}:{}:{}", self.method(), self.suffix_hex(), self.encoded_state)]
#[debug("did:{}:{}:{}", self.method(), self.suffix_hex(), self.encoded_state)]
pub struct LongFormPrismDid {
    pub suffix: Sha256Digest,
    pub encoded_state: Base64UrlStrNoPad,
}

#[enum_dispatch]
pub trait PrismDidLike {
    fn suffix(&self) -> &Sha256Digest;

    fn method(&self) -> &'static str {
        "prism"
    }

    fn suffix_hex(&self) -> HexStr {
        HexStr::from(self.suffix().as_bytes().to_owned())
    }

    fn into_canonical(self) -> CanonicalPrismDid;
}

impl PrismDidLike for CanonicalPrismDid {
    fn suffix(&self) -> &Sha256Digest {
        &self.suffix
    }

    fn into_canonical(self) -> CanonicalPrismDid {
        self
    }
}

impl PrismDidLike for LongFormPrismDid {
    fn suffix(&self) -> &Sha256Digest {
        &self.suffix
    }

    fn into_canonical(self) -> CanonicalPrismDid {
        CanonicalPrismDid { suffix: self.suffix }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DidParsingError {
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

impl CanonicalPrismDid {
    pub fn from_operation(operation: &AtalaOperation) -> Result<Self, DidParsingError> {
        Ok(LongFormPrismDid::from_operation(operation)?.into_canonical())
    }

    pub fn from_suffix_str(suffix: &str) -> Result<Self, DidParsingError> {
        let suffix = HexStr::from_str(suffix)?;
        Self::from_suffix(suffix)
    }

    pub fn from_suffix(suffix: HexStr) -> Result<Self, DidParsingError> {
        let suffix = Sha256Digest::from_bytes(&suffix.to_bytes()).map_err(DidParsingError::InvalidSuffixLength)?;
        Ok(Self { suffix })
    }
}

impl LongFormPrismDid {
    pub fn from_operation(operation: &AtalaOperation) -> Result<Self, DidParsingError> {
        match operation.operation {
            Some(Operation::CreateDid(_)) => {
                let bytes = operation.encode_to_vec();
                let suffix = sha256(bytes.clone());
                let encoded_state = Base64UrlStrNoPad::from(bytes);
                Ok(Self { suffix, encoded_state })
            }
            None => Err(DidParsingError::OperationMissing),
            Some(_) => Err(DidParsingError::InvalidOperationType(
                "operation type must be CreateDid when deriving a DID".into(),
            )),
        }
    }
}

impl FromStr for PrismDid {
    type Err = DidParsingError;

    /// # Example
    /// ```
    /// use prism_core::did::PrismDid;
    /// use std::str::FromStr;
    ///
    /// let did = PrismDid::from_str("did:prism:1234567890abcdef");
    /// assert!(did.is_err());
    ///
    /// let did = PrismDid::from_str("did:prism:0000000000000000000000000000000000000000000000000000000000000000").unwrap();
    /// assert_eq!(did.to_string(), "did:prism:0000000000000000000000000000000000000000000000000000000000000000");
    /// assert!(matches!(did, PrismDid::Canonical(_)));
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("did:prism:") {
            Err(DidParsingError::InvalidPrefix)?
        }
        let (_, s) = s.split_at("did:prism:".len());

        let canonical_match = CANONICAL_SUFFIX_RE.captures(s);
        let long_form_match = LONG_FORM_SUFFIX_RE.captures(s);

        match (canonical_match, long_form_match) {
            (None, Some(long_form_match)) => {
                let suffix: HexStr = long_form_match
                    .get(1)
                    .expect("Regex did not match this group")
                    .as_str()
                    .parse()?;
                let encoded_state: Base64UrlStrNoPad = long_form_match
                    .get(2)
                    .expect("Regex did not match this group")
                    .as_str()
                    .parse()?;
                let operation = AtalaOperation::decode(encoded_state.to_bytes().as_slice())?;
                let did = LongFormPrismDid::from_operation(&operation)?;
                if did.suffix_hex() == suffix {
                    Ok(did.into())
                } else {
                    Err(DidParsingError::UnmatchEncodedStateSuffix)
                }
            }
            (Some(canonical_match), None) => {
                let suffix: HexStr = canonical_match
                    .get(1)
                    .expect("Regex did not match this group")
                    .as_str()
                    .parse()?;
                let did = CanonicalPrismDid::from_suffix(suffix)?;
                Ok(did.into())
            }
            _ => Err(DidParsingError::UnrecognizedSuffixFormat(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DidState {
    pub did: CanonicalPrismDid,
    pub context: Vec<String>,
    pub last_operation_hash: Sha256Digest,
    pub public_keys: Vec<PublicKey>,
    pub services: Vec<Service>,
}
