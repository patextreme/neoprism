use self::operation::{ParsedPublicKey, ParsedService};
use crate::{
    crypto::{
        codec::{Base64UrlStrNoPad, HexStr},
        hash::{self, Sha256Digest},
    },
    proto::{atala_operation::Operation, AtalaOperation},
    util::MessageExt,
};
use bytes::Bytes;
use enum_dispatch::enum_dispatch;

pub mod operation;

#[enum_dispatch(PrismDid)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrismDidAny {
    Canonical(CanonicalPrismDid),
    LongForm(LongFormPrismDid),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanonicalPrismDid {
    pub suffix: Sha256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LongFormPrismDid {
    pub suffix: Sha256Digest,
    pub encoded_state: Base64UrlStrNoPad,
}

#[enum_dispatch]
pub trait PrismDid {
    fn suffix(&self) -> &Sha256Digest;

    fn method(&self) -> &str {
        "prism"
    }

    fn suffix_hex(&self) -> HexStr {
        HexStr::from(Bytes::from(self.suffix().as_bytes().to_owned()))
    }

    fn to_string(&self) -> String;

    fn to_canonical(&self) -> CanonicalPrismDid {
        CanonicalPrismDid {
            suffix: self.suffix().clone(),
        }
    }
}

impl PrismDid for CanonicalPrismDid {
    fn suffix(&self) -> &Sha256Digest {
        &self.suffix
    }

    fn to_string(&self) -> String {
        format!("did:{}:{}", self.method(), self.suffix_hex().to_string())
    }
}

impl PrismDid for LongFormPrismDid {
    fn suffix(&self) -> &Sha256Digest {
        &self.suffix
    }

    fn to_string(&self) -> String {
        format!(
            "did:{}:{}:{}",
            self.method(),
            self.suffix_hex().to_string(),
            self.encoded_state.to_string()
        )
    }
}

impl From<LongFormPrismDid> for CanonicalPrismDid {
    fn from(did: LongFormPrismDid) -> Self {
        Self { suffix: did.suffix }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum DidParsingError {
    #[error("Invalid operation type: {0}")]
    InvalidOperationType(String),
    #[error("Operation does not exist")]
    OperationMissing,
    #[error("Error when converting protobuf message to bytes: {0}")]
    EncodeError(#[from] prost::EncodeError),
    #[error("Invalid suffix length: {0}")]
    InvalidSuffixLength(String),
    #[error("Invalid suffix: {0}")]
    InvalidSuffix(#[from] hex::FromHexError),
}

impl CanonicalPrismDid {
    pub fn from_operation(operation: &AtalaOperation) -> Result<Self, DidParsingError> {
        Ok(LongFormPrismDid::from_operation(operation)?.into())
    }

    pub fn from_suffix_str(suffix: &str) -> Result<Self, DidParsingError> {
        let suffix = HexStr::try_from(suffix.to_string())?;
        Self::from_suffix(suffix)
    }

    pub fn from_suffix(suffix: HexStr) -> Result<Self, DidParsingError> {
        let bytes: Bytes = suffix.into();
        let suffix =
            Sha256Digest::from_bytes(bytes).map_err(DidParsingError::InvalidSuffixLength)?;
        Ok(Self { suffix })
    }
}

impl LongFormPrismDid {
    pub fn from_operation(operation: &AtalaOperation) -> Result<Self, DidParsingError> {
        match operation.operation {
            Some(Operation::CreateDid(_)) => {
                let bytes = operation.encode_to_bytes()?;
                let suffix = hash::sha256(bytes.clone());
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

#[derive(Debug, Clone)]
pub struct DidState {
    pub did: CanonicalPrismDid,
    pub context: Vec<String>,
    pub last_operation_hash: Sha256Digest,
    pub public_keys: Vec<ParsedPublicKey>,
    pub services: Vec<ParsedService>,
}
