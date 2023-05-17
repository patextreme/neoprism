use base64::{engine, Engine};
use bytes::Bytes;
use std::str::FromStr;

macro_rules! bytes_repr {
    ($id:ident) => {
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $id(Bytes);

        impl $id {
            pub fn as_bytes(&self) -> &[u8] {
                &self.0
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

        impl FromStr for $id {
            type Err = base64::DecodeError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let engine = $engine;
                let decoded = engine.decode(s.as_bytes())?;
                Ok(Self(decoded.into()))
            }
        }

        impl std::fmt::Display for $id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let engine = $engine;
                let encoded = engine.encode(&self.0);
                write!(f, "{}", encoded)
            }
        }

        impl std::fmt::Debug for $id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.to_string())
            }
        }
    };
}

bytes_repr!(HexStr);
bytes_repr_b64!(Base64Str, engine::general_purpose::STANDARD);
bytes_repr_b64!(Base64StrNoPad, engine::general_purpose::STANDARD_NO_PAD);
bytes_repr_b64!(Base64UrlStr, engine::general_purpose::URL_SAFE);
bytes_repr_b64!(Base64UrlStrNoPad, engine::general_purpose::URL_SAFE_NO_PAD);

impl FromStr for HexStr {
    type Err = hex::FromHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)?;
        Ok(Self(Bytes::from(bytes)))
    }
}

impl std::fmt::Debug for HexStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for HexStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}
