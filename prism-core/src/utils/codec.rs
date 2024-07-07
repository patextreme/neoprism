use std::str::FromStr;

use base64::Engine;

#[derive(Debug, thiserror::Error)]
#[error("Unable to decode {0} string repr to bytes. {1}")]
pub struct DecodeError(&'static str, String);

/// # Example
/// ```
/// use neoprism_core::utils::codec::HexStr;
/// let b = b"hello world";
/// let hexstr = HexStr::from(b);
/// assert!(hexstr.to_string() == "68656c6c6f20776f726c64");
/// ```
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, derive_more::Display, derive_more::Into, derive_more::AsRef,
)]
pub struct HexStr(String);

impl HexStr {
    pub fn to_bytes(&self) -> Vec<u8> {
        hex::decode(self.as_ref()).unwrap_or_else(|_| {
            unreachable!(
                "{} should be a valid hex string",
                std::any::type_name::<Self>()
            )
        })
    }
}

impl<B: AsRef<[u8]>> From<B> for HexStr {
    fn from(value: B) -> Self {
        Self(hex::encode(value.as_ref()))
    }
}

impl FromStr for HexStr {
    type Err = DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)
            .map_err(|e| DecodeError(std::any::type_name::<Self>(), e.to_string()))?;
        Ok(bytes.as_slice().into())
    }
}

/// # Example
/// ```
/// use neoprism_core::utils::codec::Base64UrlStr;
/// let b = b"hello world";
/// let b64 = Base64UrlStr::from(b);
/// assert!(b64.to_string() == "aGVsbG8gd29ybGQ=");
/// ```
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, derive_more::Display, derive_more::Into, derive_more::AsRef,
)]
pub struct Base64UrlStr(String);

impl Base64UrlStr {
    pub fn to_bytes(&self) -> Vec<u8> {
        base64::engine::general_purpose::URL_SAFE
            .decode(self.as_ref().as_bytes())
            .unwrap_or_else(|_| {
                unreachable!(
                    "{} should be a valid base64 string",
                    std::any::type_name::<Self>()
                )
            })
    }
}

impl<B: AsRef<[u8]>> From<B> for Base64UrlStr {
    fn from(value: B) -> Self {
        Self(base64::engine::general_purpose::URL_SAFE.encode(value.as_ref()))
    }
}

impl FromStr for Base64UrlStr {
    type Err = DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let engine = base64::engine::general_purpose::URL_SAFE;
        let bytes = engine
            .decode(s.as_bytes())
            .map_err(|e| DecodeError(std::any::type_name::<Self>(), e.to_string()))?;
        Ok(bytes.as_slice().into())
    }
}

/// # Example
/// ```
/// use neoprism_core::utils::codec::Base64UrlStrNoPad;
/// let b = b"hello world";
/// let b64 = Base64UrlStrNoPad::from(b);
/// assert!(b64.to_string() == "aGVsbG8gd29ybGQ");
/// ```
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, derive_more::Display, derive_more::Into, derive_more::AsRef,
)]
pub struct Base64UrlStrNoPad(String);

impl Base64UrlStrNoPad {
    pub fn to_bytes(&self) -> Vec<u8> {
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(self.as_ref().as_bytes())
            .unwrap_or_else(|_| {
                unreachable!(
                    "{} should be a valid base64 string",
                    std::any::type_name::<Self>()
                )
            })
    }
}

impl<B: AsRef<[u8]>> From<B> for Base64UrlStrNoPad {
    fn from(value: B) -> Self {
        Self(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(value.as_ref()))
    }
}

impl FromStr for Base64UrlStrNoPad {
    type Err = DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let bytes = engine
            .decode(s.as_bytes())
            .map_err(|e| DecodeError(std::any::type_name::<Self>(), e.to_string()))?;
        Ok(bytes.as_slice().into())
    }
}
