mod ssi;
mod storage;

use std::str::FromStr;

use identus_apollo::hex::HexStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
pub use ssi::*;
pub use storage::*;

use crate::prelude::SignedPrismOperation;
use crate::proto::MessageExt;

#[derive(Debug, Clone)]
pub struct OperationParameters {
    pub max_services: usize,
    pub max_public_keys: usize,
    pub max_id_size: usize,
    pub max_type_size: usize,
    pub max_service_endpoint_size: usize,
}

impl OperationParameters {
    pub fn v1() -> Self {
        Self {
            max_services: 50,
            max_public_keys: 50,
            max_id_size: 50,
            max_type_size: 100,
            max_service_endpoint_size: 300,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, derive_more::From, derive_more::Into)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(description = "A hexadecimal string representing a SignedPrismOperation", value_type = String, example = "0a086d61737465722d30124630440220442eec28ec60464acd8df155e73f88a1c7faf4549975582ff0601449525aba31022019257250071818066b377b83a8b1765df1b7dc21d9bccfc7d5da036801d3ba0e1a420a400a3e123c0a086d61737465722d3010014a2e0a09736563703235366b3112210398e61c14328a6a844eec6dc084b825ae8525f10204e9244aaf61260bd221a457"))]
pub struct SignedPrismOperationHexStr(
    #[serde(
        serialize_with = "SignedPrismOperationHexStr::serialize",
        deserialize_with = "SignedPrismOperationHexStr::deserialize"
    )]
    SignedPrismOperation,
);

impl SignedPrismOperationHexStr {
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
