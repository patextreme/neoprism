use std::str::FromStr;

use base64::Engine;

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum Error {
    #[display("unable to base64 decode '{value}' to type {type_name}")]
    DecodeBase64 {
        source: base64::DecodeError,
        type_name: &'static str,
        value: String,
    },
}

/// # Example
/// ```
/// use prism_core::utils::codec::Base64UrlStr;
///
/// let b = b"hello world";
/// let b64 = Base64UrlStr::from(b);
/// assert!(b64.to_string() == "aGVsbG8gd29ybGQ=");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, derive_more::Display, derive_more::Into, derive_more::AsRef)]
pub struct Base64UrlStr(String);

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
/// use prism_core::utils::codec::{Base64UrlStr, Error};
///
/// let b64 = Base64UrlStr::from_str("aGVsbG8gd29ybGQ=").unwrap();
/// assert_eq!(b64, Base64UrlStr::from(b"hello world"));
///
/// let err = Base64UrlStr::from_str("invalid").err().unwrap();
/// assert!(matches!(err, Error::DecodeBase64 { .. }));
/// ```
impl FromStr for Base64UrlStr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let engine = base64::engine::general_purpose::URL_SAFE;
        let bytes = engine.decode(s.as_bytes()).map_err(|e| Error::DecodeBase64 {
            source: e,
            type_name: std::any::type_name::<Self>(),
            value: s.to_string(),
        })?;
        Ok(bytes.as_slice().into())
    }
}

/// # Example
/// ```
/// use prism_core::utils::codec::Base64UrlStrNoPad;
///
/// let b = b"hello world";
/// let b64 = Base64UrlStrNoPad::from(b);
/// assert!(b64.to_string() == "aGVsbG8gd29ybGQ");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, derive_more::Display, derive_more::Into, derive_more::AsRef)]
pub struct Base64UrlStrNoPad(String);

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
/// use prism_core::utils::codec::{Base64UrlStrNoPad, Error};
///
/// let b64 = Base64UrlStrNoPad::from_str("aGVsbG8gd29ybGQ").unwrap();
/// assert_eq!(b64, Base64UrlStrNoPad::from(b"hello world"));
///
/// let err = Base64UrlStrNoPad::from_str("invalid").err().unwrap();
/// assert!(matches!(err, Error::DecodeBase64 { .. }));
/// ```
impl FromStr for Base64UrlStrNoPad {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let bytes = engine.decode(s.as_bytes()).map_err(|e| Error::DecodeBase64 {
            source: e,
            type_name: std::any::type_name::<Self>(),
            value: s.to_string(),
        })?;
        Ok(bytes.as_slice().into())
    }
}
