#[cfg(feature = "ed25519")]
pub mod ed25519;
#[cfg(feature = "secp256k1")]
pub mod secp256k1;
#[cfg(feature = "x25519")]
pub mod x25519;

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
    #[cfg(feature = "ed25519")]
    #[from]
    #[display("unable to parse Ed25519 key")]
    Ed25519KeyParsing { source: ed25519_dalek::SignatureError },
    #[cfg(feature = "secp256k1")]
    #[from]
    #[display("unable to parse secp256k1 key")]
    Secp256k1KeyParsing { source: ::k256::elliptic_curve::Error },
}

pub trait ToPublicKey<Pk> {
    fn to_public_key(&self) -> Result<Pk, Error>;
}
