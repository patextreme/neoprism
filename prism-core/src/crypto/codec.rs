use base64::{engine, Engine};
use bytes::Bytes;

macro_rules! bytes_repr {
    ($id:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $id(Bytes);

        impl $id {
            pub fn as_bytes(&self) -> &[u8] {
                &self.0
            }

            pub fn to_string(&self) -> String {
                String::from(self)
            }
        }

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

macro_rules! bytes_repr_b64 {
    ($id:ident, $engine:expr) => {
        bytes_repr!($id);

        impl TryFrom<String> for $id {
            type Error = base64::DecodeError;
            fn try_from(s: String) -> Result<Self, Self::Error> {
                let engine = $engine;
                let decoded = engine.decode(s.as_bytes())?;
                Ok(Self(decoded.into()))
            }
        }

        impl From<$id> for String {
            fn from(s: $id) -> Self {
                let engine = $engine;
                engine.encode(s.0)
            }
        }

        impl From<&$id> for String {
            fn from(s: &$id) -> Self {
                let engine = $engine;
                engine.encode(&s.0)
            }
        }
    };
}

bytes_repr!(HexStr);
bytes_repr_b64!(Base64Str, engine::general_purpose::STANDARD);
bytes_repr_b64!(Base64StrNoPad, engine::general_purpose::STANDARD_NO_PAD);
bytes_repr_b64!(Base64UrlStr, engine::general_purpose::URL_SAFE);
bytes_repr_b64!(Base64UrlStrNoPad, engine::general_purpose::URL_SAFE_NO_PAD);

impl TryFrom<String> for HexStr {
    type Error = hex::FromHexError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        let bytes = hex::decode(s)?;
        Ok(Self(Bytes::from(bytes)))
    }
}

impl From<HexStr> for String {
    fn from(s: HexStr) -> Self {
        hex::encode(s.0)
    }
}

impl From<&HexStr> for String {
    fn from(s: &HexStr) -> Self {
        hex::encode(&s.0)
    }
}
