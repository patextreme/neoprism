use std::str::FromStr;

use identus_apollo::hex::HexStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, derive_more::From, ToSchema)]
#[schema(description = "A hexadecimal string representing underlying bytes", value_type = String, example = "0123456789abcdef")]
pub struct HexStrBytes(
    #[serde(
        serialize_with = "HexStrBytes::serialize",
        deserialize_with = "HexStrBytes::deserialize"
    )]
    Vec<u8>,
);

impl HexStrBytes {
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
