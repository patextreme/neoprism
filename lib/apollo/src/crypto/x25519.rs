use super::{EncodeArray, EncodeVec, Error, ToPublicKey};
use crate::base64::Base64UrlStrNoPad;
use crate::jwk::{EncodeJwk, Jwk};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct X25519PublicKey(x25519_dalek::PublicKey);

impl EncodeVec for X25519PublicKey {
    fn encode_vec(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}

impl EncodeArray<32> for X25519PublicKey {
    fn encode_array(&self) -> [u8; 32] {
        self.0.to_bytes()
    }
}

impl<T: AsRef<[u8]>> ToPublicKey<X25519PublicKey> for T {
    fn to_public_key(&self) -> Result<X25519PublicKey, Error> {
        let slice = self.as_ref();
        let Some((key, _)) = slice.split_first_chunk::<32>() else {
            Err(Error::InvalidKeySize {
                expected: 32,
                actual: slice.len(),
                key_type: std::any::type_name::<X25519PublicKey>(),
            })?
        };
        let key = x25519_dalek::PublicKey::from(key.to_owned());
        Ok(X25519PublicKey(key))
    }
}

impl EncodeJwk for X25519PublicKey {
    fn encode_jwk(&self) -> Jwk {
        let x = self.encode_array();
        Jwk {
            kty: "OKP".to_string(),
            crv: "X25519".to_string(),
            x: Some(Base64UrlStrNoPad::from(x)),
            y: None,
        }
    }
}
