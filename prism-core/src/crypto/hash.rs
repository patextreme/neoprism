use super::codec::HexStr;
use bytes::Bytes;
use ring::digest;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Sha256Digest([u8; 32]);

impl std::fmt::Debug for Sha256Digest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = Bytes::copy_from_slice(&self.0);
        HexStr::from(bytes).fmt(f)
    }
}

impl Sha256Digest {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Convert bytes to the digest.
    /// This only validate and wrap the raw bytes, it does not hash.
    ///
    /// # Example
    ///
    /// ```
    /// use prism_core::crypto::hash::Sha256Digest;
    /// let digest = Sha256Digest::from_bytes(vec![42u8; 32].into()).unwrap();
    /// assert_eq!(digest.as_bytes(), vec![42u8; 32]);
    ///
    /// let digest = Sha256Digest::from_bytes(vec![42u8; 31].into());
    /// assert!(digest.is_err());
    /// ```
    pub fn from_bytes(bytes: Bytes) -> Result<Self, String> {
        if bytes.len() != 32 {
            return Err(format!("Expected 32 bytes, got {} bytes", bytes.len()));
        }

        let mut digest = [0u8; 32];
        digest.copy_from_slice(&bytes);
        Ok(Self(digest))
    }
}

impl From<Sha256Digest> for Bytes {
    fn from(digest: Sha256Digest) -> Self {
        digest.0.to_vec().into()
    }
}

pub fn sha256<B: AsRef<[u8]>>(bytes: B) -> Sha256Digest {
    let digest = digest::digest(&digest::SHA256, bytes.as_ref());
    let digest: [u8; 32] = digest
        .as_ref()
        .try_into()
        .expect("The digest must have length of 32 bytes");
    Sha256Digest(digest)
}
