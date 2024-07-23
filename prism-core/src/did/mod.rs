use std::str::FromStr;
use std::sync::OnceLock;

use enum_dispatch::enum_dispatch;
use prost::Message;
use regex::Regex;

use self::operation::{PublicKey, Service};
use crate::proto::atala_operation::Operation;
use crate::proto::AtalaOperation;
use crate::utils::codec::{Base64UrlStrNoPad, HexStr};
use crate::utils::hash::{sha256, Sha256Digest};

pub mod operation;

static CANONICAL_SUFFIX_RE: OnceLock<Regex> = OnceLock::new();
static LONG_FORM_SUFFIX_RE: OnceLock<Regex> = OnceLock::new();

#[enum_dispatch(PrismDidLike, Display)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrismDid {
    Canonical(CanonicalPrismDid),
    LongForm(LongFormPrismDid),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CanonicalPrismDid {
    pub suffix: Sha256Digest,
}

#[derive(Clone, PartialEq, Eq, Hash)]
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

    fn to_canonical(&self) -> CanonicalPrismDid {
        CanonicalPrismDid {
            suffix: self.suffix().clone(),
        }
    }
}

impl PrismDidLike for CanonicalPrismDid {
    fn suffix(&self) -> &Sha256Digest {
        &self.suffix
    }
}

impl PrismDidLike for LongFormPrismDid {
    fn suffix(&self) -> &Sha256Digest {
        &self.suffix
    }
}

impl From<LongFormPrismDid> for CanonicalPrismDid {
    fn from(did: LongFormPrismDid) -> Self {
        Self { suffix: did.suffix }
    }
}

impl std::fmt::Display for CanonicalPrismDid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "did:{}:{}", self.method(), self.suffix_hex())
    }
}

impl std::fmt::Display for LongFormPrismDid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "did:{}:{}:{}", self.method(), self.suffix_hex(), self.encoded_state)
    }
}

impl std::fmt::Debug for CanonicalPrismDid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Debug for LongFormPrismDid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
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
    InvalidSuffix(#[from] crate::utils::codec::DecodeError),
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
        Ok(LongFormPrismDid::from_operation(operation)?.into())
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
        let canonical_did_re = CANONICAL_SUFFIX_RE
            .get_or_init(|| Regex::new(r"^([0-9a-f]{64}$)").expect("CANONICAL_SUFFIX_RE regex is invalid"));
        let long_form_did_re = LONG_FORM_SUFFIX_RE.get_or_init(|| {
            Regex::new(r"^([0-9a-f]{64}):([A-Za-z0-9_-]+$)").expect("LONG_FORM_SUFFIX_RE regex is invalid")
        });

        if !s.starts_with("did:prism:") {
            Err(DidParsingError::InvalidPrefix)?
        }
        let (_, s) = s.split_at("did:prism:".len());

        let canonical_match = canonical_did_re.captures(s);
        let long_form_match = long_form_did_re.captures(s);

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
