use std::backtrace::Backtrace;

pub mod ed25519;
pub mod secp256k1;
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

#[derive(Debug, thiserror::Error)]
pub enum ToPublicKeyError {
    #[error("Invalid key size: expected {expected}, actual {actual}")]
    InvalidKeySize {
        expected: usize,
        actual: usize,
        backtrace: Backtrace,
    },
    #[error("Ed25519 key is invalid. {source}")]
    Ed25519Signature {
        #[from]
        source: ed25519_dalek::SignatureError,
        backtrace: Backtrace,
    },
    #[error("Secp256k1 key is invalid. {source}")]
    Secp256k1Signature {
        #[from]
        source: ::secp256k1::Error,
        backtrace: Backtrace,
    },
}

pub trait ToPublicKey<Pk> {
    fn to_public_key(&self) -> Result<Pk, ToPublicKeyError>;
}
