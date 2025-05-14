use crate::base64::Base64UrlStrNoPad;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Jwk {
    pub kty: String,
    pub crv: String,
    pub x: Option<Base64UrlStrNoPad>,
    pub y: Option<Base64UrlStrNoPad>,
}

pub trait EncodeJwk {
    fn encode_jwk(&self) -> Jwk;
}
