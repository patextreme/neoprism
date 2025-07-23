use std::str::FromStr;

use identus_apollo::hex::HexStr;
use identus_did_prism::did::CanonicalPrismDid;
use lazybe::macros::Newtype;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

mod indexer;
mod submitter;

pub use indexer::*;
pub use submitter::*;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, Newtype, derive_more::From)]
pub struct DidSuffix(Vec<u8>);

impl From<CanonicalPrismDid> for DidSuffix {
    fn from(value: CanonicalPrismDid) -> Self {
        value.suffix.to_vec().into()
    }
}

impl TryFrom<DidSuffix> for CanonicalPrismDid {
    type Error = crate::Error;

    fn try_from(value: DidSuffix) -> Result<Self, Self::Error> {
        let suffix = HexStr::from(value.0);
        let did = CanonicalPrismDid::from_suffix(suffix)?;
        Ok(did)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Newtype, derive_more::From, ToSchema)]
#[schema(description = "A hexadecimal string representing underlying bytes", value_type = String, example = "0123456789abcdef")]
pub struct BytesHex(
    #[serde(serialize_with = "BytesHex::serialize", deserialize_with = "BytesHex::deserialize")] Vec<u8>,
);

impl BytesHex {
    fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_str = HexStr::from(bytes);
        serializer.serialize_str(&hex_str.to_string())
    }

    fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_str = String::deserialize(deserializer)?;
        let bytes = HexStr::from_str(&hex_str)
            .map_err(|e| serde::de::Error::custom(format!("Value is not a valid hex: {e}")))?;
        Ok(bytes.to_bytes().into())
    }
}
