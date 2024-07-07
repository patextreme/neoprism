use std::str::FromStr;

use self::operation::{PublicKey, Service};
use crate::{
    proto::{atala_operation::Operation, AtalaOperation},
    utils::{
        codec::{Base64UrlStrNoPad, HexStr},
        hash::{sha256, Sha256Digest},
    },
};
use enum_dispatch::enum_dispatch;
use prost::Message;

pub mod operation;

#[enum_dispatch(PrismDid)]
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

impl std::fmt::Display for PrismDid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrismDid::Canonical(did) => did.fmt(f),
            PrismDid::LongForm(did) => did.fmt(f),
        }
    }
}

impl std::fmt::Display for CanonicalPrismDid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "did:{}:{}", self.method(), self.suffix_hex())
    }
}

impl std::fmt::Display for LongFormPrismDid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "did:{}:{}:{}",
            self.method(),
            self.suffix_hex(),
            self.encoded_state
        )
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
        let suffix = Sha256Digest::from_bytes(&suffix.to_bytes())
            .map_err(DidParsingError::InvalidSuffixLength)?;
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
                Ok(Self {
                    suffix,
                    encoded_state,
                })
            }
            None => Err(DidParsingError::OperationMissing),
            Some(_) => Err(DidParsingError::InvalidOperationType(
                "operation type must be CreateDid when deriving a DID".into(),
            )),
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
