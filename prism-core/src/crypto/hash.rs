use bytes::Bytes;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sha256Digest([u8; 32]);

impl From<Sha256Digest> for Bytes {
    fn from(digest: Sha256Digest) -> Self {
        digest.0.to_vec().into()
    }
}

pub fn sha_256<B: Into<Bytes>>(bytes: B) -> Sha256Digest {
    let bytes: Bytes = bytes.into();
    let bytes: Vec<u8> = bytes.into();
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize().into();
    Sha256Digest(digest)
}
