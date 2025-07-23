use std::str::FromStr;

use identus_apollo::hex::HexStr;
use identus_did_prism::prelude::SignedPrismOperation;
use identus_did_prism::proto::MessageExt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, derive_more::From, derive_more::Into, ToSchema)]
#[schema(description = "A hexadecimal string representing underlying bytes", value_type = String, example = "0a086d61737465722d30124630440220442eec28ec60464acd8df155e73f88a1c7faf4549975582ff0601449525aba31022019257250071818066b377b83a8b1765df1b7dc21d9bccfc7d5da036801d3ba0e1a420a400a3e123c0a086d61737465722d3010014a2e0a09736563703235366b3112210398e61c14328a6a844eec6dc084b825ae8525f10204e9244aaf61260bd221a457")]
pub struct SignedOperationHexStr(
    #[serde(
        serialize_with = "SignedOperationHexStr::serialize",
        deserialize_with = "SignedOperationHexStr::deserialize"
    )]
    SignedPrismOperation,
);

impl SignedOperationHexStr {
    fn serialize<S>(op: &SignedPrismOperation, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_str = HexStr::from(&op.encode_to_vec());
        serializer.serialize_str(&hex_str.to_string())
    }

    fn deserialize<'de, D>(deserializer: D) -> Result<SignedPrismOperation, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_str = String::deserialize(deserializer)?;
        let bytes = HexStr::from_str(&hex_str)
            .map_err(|e| serde::de::Error::custom(format!("Value is not a valid hex: {e}")))?;
        let op = SignedPrismOperation::decode(&bytes.to_bytes())
            .map_err(|e| serde::de::Error::custom(format!("Value cannot be decoded to SignedPrismOperation: {e}")))?;
        Ok(op)
    }
}
