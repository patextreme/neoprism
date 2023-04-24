use base64::{engine, Engine};
use bytes::Bytes;

macro_rules! bytes_repr {
    ($id:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $id(Bytes);

        impl From<Bytes> for $id {
            fn from(bytes: Bytes) -> Self {
                Self(bytes)
            }
        }

        impl From<$id> for Bytes {
            fn from(s: $id) -> Self {
                s.0
            }
        }

        impl From<&$id> for Bytes {
            fn from(s: &$id) -> Self {
                s.0.clone()
            }
        }
    };
}

bytes_repr!(HexStr);
bytes_repr!(Base64Str);
bytes_repr!(Base64StrNoPad);
bytes_repr!(Base64UrlStr);
bytes_repr!(Base64UrlStrNoPad);

impl TryFrom<&str> for HexStr {
    type Error = hex::FromHexError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let bytes = hex::decode(s)?;
        Ok(Self(Bytes::from(bytes)))
    }
}

impl From<HexStr> for String {
    fn from(s: HexStr) -> Self {
        hex::encode(s.0)
    }
}

impl TryFrom<&str> for Base64Str {
    type Error = base64::DecodeError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let engine = engine::general_purpose::STANDARD;
        let decoded = engine.decode(s.as_bytes())?;
        Ok(Self(decoded.into()))
    }
}

impl From<Base64Str> for String {
    fn from(s: Base64Str) -> Self {
        let engine = engine::general_purpose::STANDARD;
        engine.encode(s.0)
    }
}

impl TryFrom<&str> for Base64StrNoPad {
    type Error = base64::DecodeError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let engine = engine::general_purpose::STANDARD_NO_PAD;
        let decoded = engine.decode(s.as_bytes())?;
        Ok(Self(decoded.into()))
    }
}

impl From<Base64StrNoPad> for String {
    fn from(s: Base64StrNoPad) -> Self {
        let engine = engine::general_purpose::STANDARD_NO_PAD;
        engine.encode(s.0)
    }
}

impl TryFrom<&str> for Base64UrlStr {
    type Error = base64::DecodeError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let engine = engine::general_purpose::URL_SAFE;
        let decoded = engine.decode(s.as_bytes())?;
        Ok(Self(decoded.into()))
    }
}

impl From<Base64UrlStr> for String {
    fn from(s: Base64UrlStr) -> Self {
        let engine = engine::general_purpose::URL_SAFE;
        engine.encode(s.0)
    }
}

impl TryFrom<&str> for Base64UrlStrNoPad {
    type Error = base64::DecodeError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let engine = engine::general_purpose::URL_SAFE_NO_PAD;
        let decoded = engine.decode(s.as_bytes())?;
        Ok(Self(decoded.into()))
    }
}

impl From<Base64UrlStrNoPad> for String {
    fn from(s: Base64UrlStrNoPad) -> Self {
        let engine = engine::general_purpose::URL_SAFE_NO_PAD;
        engine.encode(s.0)
    }
}
