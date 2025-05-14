use std::str::FromStr;

use base64::Engine;

#[derive(Debug, derive_more::Display, derive_more::Error)]
#[display("unable to base64 decode '{value}' to type {type_name}")]
pub struct Error {
    source: base64::DecodeError,
    type_name: &'static str,
    value: String,
}

/// # Example
/// ```
/// use apollo::base64::Base64UrlStr;
///
/// let b = b"hello world";
/// let b64 = Base64UrlStr::from(b);
/// assert!(b64.to_string() == "aGVsbG8gd29ybGQ=");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, derive_more::Display, derive_more::Into, derive_more::AsRef)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Base64UrlStr(
    #[cfg_attr(feature = "serde", serde(deserialize_with = "serde_impl::deserialize_base64_url"))] String,
);

impl Base64UrlStr {
    pub fn to_bytes(&self) -> Vec<u8> {
        base64::engine::general_purpose::URL_SAFE
            .decode(self.as_ref().as_bytes())
            .unwrap_or_else(|_| unreachable!("{} should be a valid base64 string", std::any::type_name::<Self>()))
    }
}

impl<B: AsRef<[u8]>> From<B> for Base64UrlStr {
    fn from(value: B) -> Self {
        Self(base64::engine::general_purpose::URL_SAFE.encode(value.as_ref()))
    }
}

/// # Example
/// ```
/// use std::str::FromStr;
///
/// use apollo::base64::{Base64UrlStr, Error};
///
/// let b64 = Base64UrlStr::from_str("aGVsbG8gd29ybGQ=").unwrap();
/// assert_eq!(b64, Base64UrlStr::from(b"hello world"));
///
/// let b64 = Base64UrlStr::from_str("invalid");
/// assert!(b64.is_err());
/// ```
impl FromStr for Base64UrlStr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let engine = base64::engine::general_purpose::URL_SAFE;
        let bytes = engine.decode(s.as_bytes()).map_err(|e| Error {
            source: e,
            type_name: std::any::type_name::<Self>(),
            value: s.to_string(),
        })?;
        Ok(bytes.as_slice().into())
    }
}

/// # Example
/// ```
/// use apollo::base64::Base64UrlStrNoPad;
///
/// let b = b"hello world";
/// let b64 = Base64UrlStrNoPad::from(b);
/// assert!(b64.to_string() == "aGVsbG8gd29ybGQ");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, derive_more::Display, derive_more::Into, derive_more::AsRef)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Base64UrlStrNoPad(
    #[cfg_attr(
        feature = "serde",
        serde(deserialize_with = "serde_impl::deserialize_base64_url_no_pad")
    )]
    String,
);

impl Base64UrlStrNoPad {
    pub fn to_bytes(&self) -> Vec<u8> {
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(self.as_ref().as_bytes())
            .unwrap_or_else(|_| unreachable!("{} should be a valid base64 string", std::any::type_name::<Self>()))
    }
}

impl<B: AsRef<[u8]>> From<B> for Base64UrlStrNoPad {
    fn from(value: B) -> Self {
        Self(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(value.as_ref()))
    }
}

/// # Example
/// ```
/// use std::str::FromStr;
///
/// use apollo::base64::{Base64UrlStrNoPad, Error};
///
/// let b64 = Base64UrlStrNoPad::from_str("aGVsbG8gd29ybGQ").unwrap();
/// assert_eq!(b64, Base64UrlStrNoPad::from(b"hello world"));
///
/// let b64 = Base64UrlStrNoPad::from_str("invalid");
/// assert!(b64.is_err());
/// ```
impl FromStr for Base64UrlStrNoPad {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let bytes = engine.decode(s.as_bytes()).map_err(|e| Error {
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

    use super::{Base64UrlStr, Base64UrlStrNoPad};

    pub fn deserialize_base64_url<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Base64UrlStr::from_str(&raw)
            .map(|i| i.to_string())
            .map_err(|e| serde::de::Error::custom(e.to_string()))
    }

    pub fn deserialize_base64_url_no_pad<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Base64UrlStrNoPad::from_str(&raw)
            .map(|i| i.to_string())
            .map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}
