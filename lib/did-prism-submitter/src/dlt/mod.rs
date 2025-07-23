use std::str::FromStr;

use identus_apollo::hash::Sha256Digest;
use identus_apollo::hex::HexStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "cardano-wallet")]
pub mod cardano_wallet;

#[derive(
    Clone, PartialEq, Eq, Hash, Serialize, Deserialize, derive_more::Debug, derive_more::Display, derive_more::From,
)]
#[display("{}", identus_apollo::hex::HexStr::from(self.0.as_bytes()))]
#[debug("{}", identus_apollo::hex::HexStr::from(self.0.as_bytes()))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(value_type = String, example = "5ab0cf7e4c7cd4b63ba84a4fe299409be12ba85607cb6d1a149e80bc2eac070d"))]
pub struct TxId(#[serde(serialize_with = "TxId::serialize", deserialize_with = "TxId::deserialize")] Sha256Digest);

impl TxId {
    fn serialize<S>(bytes: &Sha256Digest, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_str = HexStr::from(bytes.as_bytes());
        serializer.serialize_str(&hex_str.to_string())
    }

    fn deserialize<'de, D>(deserializer: D) -> Result<Sha256Digest, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_str = String::deserialize(deserializer)?;
        let bytes = HexStr::from_str(&hex_str)
            .map_err(|e| serde::de::Error::custom(format!("Value is not a valid hex: {e}")))?;
        let digest = Sha256Digest::from_bytes(&bytes.to_bytes())
            .map_err(|e| serde::de::Error::custom(format!("Value is not a valid digest: {e}")))?;
        Ok(digest)
    }
}
