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
/// use apollo::hex::HexStr;
///
/// let b = b"hello world";
/// let hexstr = HexStr::from(b);
/// assert!(hexstr.to_string() == "68656c6c6f20776f726c64");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, derive_more::Display, derive_more::Into, derive_more::AsRef)]
pub struct HexStr(String);

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
/// use apollo::hex::{Error, HexStr};
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
