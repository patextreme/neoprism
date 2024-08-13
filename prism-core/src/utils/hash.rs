use ring::digest;

#[derive(Debug, Clone, PartialEq, Eq, Hash, derive_more::From)]
pub struct Sha256Digest([u8; 32]);

impl Sha256Digest {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn as_array(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    /// Convert bytes to the digest.
    /// This only validate and wrap the raw bytes, it does not hash.
    ///
    /// # Example
    ///
    /// ```
    /// use prism_core::utils::hash::Sha256Digest;
    /// let digest = Sha256Digest::from_bytes(&vec![42u8; 32]).unwrap();
    /// assert_eq!(digest.as_bytes(), vec![42u8; 32]);
    ///
    /// let digest = Sha256Digest::from_bytes(&vec![42u8; 31]);
    /// assert!(digest.is_err());
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != 32 {
            return Err(format!("Expected 32 bytes, got {} bytes", bytes.len()));
        }

        let mut digest = [0u8; 32];
        digest.copy_from_slice(bytes);
        Ok(Self(digest))
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
