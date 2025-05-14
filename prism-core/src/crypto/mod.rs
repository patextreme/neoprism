use apollo::base64::Base64UrlStrNoPad;
use enum_dispatch::enum_dispatch;

pub mod ed25519;
pub mod secp256k1;
pub mod x25519;

pub struct Jwk {
    pub kty: String,
    pub crv: String,
    pub x: Option<Base64UrlStrNoPad>,
    pub y: Option<Base64UrlStrNoPad>,
}

#[enum_dispatch]
pub trait EncodeJwk {
    fn encode_jwk(&self) -> Jwk;
}

#[enum_dispatch]
pub trait EncodeVec {
    fn encode_vec(&self) -> Vec<u8>;
}

pub trait EncodeArray<const N: usize> {
    fn encode_array(&self) -> [u8; N];
}

pub trait Verifiable {
    fn verify(&self, message: &[u8], signature: &[u8]) -> bool;
}

#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum Error {
    #[display("expected {key_type} key size to be {expected}, got size {actual}")]
    InvalidKeySize {
        expected: usize,
        actual: usize,
        key_type: &'static str,
    },
    #[from]
    #[display("unable to parse Ed25519 key")]
    Ed25519KeyParsing { source: ed25519_dalek::SignatureError },
    #[from]
    #[display("unable to parse secp256k1 key")]
    Secp256k1KeyParsing { source: ::k256::elliptic_curve::Error },
    #[display("unsupported curve {curve}")]
    UnsupportedCurve { curve: String },
}

pub trait ToPublicKey<Pk> {
    fn to_public_key(&self) -> Result<Pk, Error>;
}
