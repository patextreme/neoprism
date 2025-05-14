use apollo::base64::Base64UrlStrNoPad;
use apollo::jwk::{EncodeJwk, Jwk};

use super::{EncodeArray, EncodeVec, Error, ToPublicKey, Verifiable};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ed25519PublicKey(ed25519_dalek::VerifyingKey);

impl EncodeVec for Ed25519PublicKey {
    fn encode_vec(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }
}

impl EncodeArray<32> for Ed25519PublicKey {
    fn encode_array(&self) -> [u8; 32] {
        self.0.to_bytes()
    }
}

impl Verifiable for Ed25519PublicKey {
    fn verify(&self, message: &[u8], signature: &[u8]) -> bool {
        let Ok(signature) = ed25519_dalek::Signature::from_slice(signature) else {
            return false;
        };
        self.0.verify_strict(message, &signature).is_ok()
    }
}

impl<T: AsRef<[u8]>> ToPublicKey<Ed25519PublicKey> for T {
    fn to_public_key(&self) -> Result<Ed25519PublicKey, Error> {
        let slice = self.as_ref();
        let Some((key, _)) = slice.split_first_chunk::<32>() else {
            Err(Error::InvalidKeySize {
                expected: 32,
                actual: slice.len(),
                key_type: std::any::type_name::<Ed25519PublicKey>(),
            })?
        };
        let key = ed25519_dalek::VerifyingKey::from_bytes(key)?;
        Ok(Ed25519PublicKey(key))
    }
}

impl EncodeJwk for Ed25519PublicKey {
    fn encode_jwk(&self) -> Jwk {
        let x = self.encode_array();
        Jwk {
            kty: "OKP".to_string(),
            crv: "Ed25519".to_string(),
            x: Some(Base64UrlStrNoPad::from(x)),
            y: None,
        }
    }
}
