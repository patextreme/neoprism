use std::str::FromStr;

#[derive(Debug, derive_more::Display, derive_more::Error)]
#[display("unable to hex decode '{value}' to type {type_name}")]
pub struct Error {
    source: hex::FromHexError,
    type_name: &'static str,
    value: String,
}

/// # Example
/// ```
/// use identus_apollo::hex::HexStr;
///
/// let b = b"hello world";
/// let hexstr = HexStr::from(b);
/// assert!(hexstr.to_string() == "68656c6c6f20776f726c64");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, derive_more::Display, derive_more::Into, derive_more::AsRef)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct HexStr(#[cfg_attr(feature = "serde", serde(deserialize_with = "serde_impl::deserialize_hex"))] String);

impl HexStr {
    pub fn to_bytes(&self) -> Vec<u8> {
        hex::decode(self.as_ref())
            .unwrap_or_else(|_| unreachable!("{} should be a valid hex string", std::any::type_name::<Self>()))
    }
}

impl<B: AsRef<[u8]>> From<B> for HexStr {
    fn from(value: B) -> Self {
        Self(hex::encode(value.as_ref()))
    }
}

/// # Example
/// ```
/// use std::str::FromStr;
///
/// use identus_apollo::hex::{Error, HexStr};
///
/// let hexstr = HexStr::from_str("68656c6c6f20776f726c64").unwrap();
/// assert_eq!(hexstr, HexStr::from(b"hello world"));
///
/// let hexstr = HexStr::from_str("invalid");
/// assert!(hexstr.is_err());
/// ```
impl FromStr for HexStr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|e| Error {
            source: e,
            type_name: std::any::type_name::<Self>(),
            value: s.to_string(),
        })?;
        Ok(bytes.as_slice().into())
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use std::str::FromStr;

    use serde::{Deserialize, Deserializer};

    use super::HexStr;

    pub fn deserialize_hex<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        HexStr::from_str(&raw)
            .map(|i| i.to_string())
            .map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}
