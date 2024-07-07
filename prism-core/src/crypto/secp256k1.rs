use std::fmt::Debug;

use super::{EncodeArray, EncodeVec, ToPublicKey, ToPublicKeyError, Verifiable};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Secp256k1PublicKey(secp256k1::PublicKey);

impl Secp256k1PublicKey {
    pub fn random() -> Self {
        let secp = secp256k1::Secp256k1::new();
        let (_, pk) = secp.generate_keypair(&mut rand::thread_rng());
        Self(pk)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CurvePoint {
    pub x: [u8; 32],
    pub y: [u8; 32],
}

impl EncodeVec for Secp256k1PublicKey {
    fn encode_vec(&self) -> Vec<u8> {
        self.encode_compressed().into()
    }
}

impl EncodeArray<33> for Secp256k1PublicKey {
    fn encode_array(&self) -> [u8; 33] {
        self.encode_compressed()
    }
}

impl EncodeArray<65> for Secp256k1PublicKey {
    fn encode_array(&self) -> [u8; 65] {
        self.encode_uncompressed()
    }
}

impl Verifiable for Secp256k1PublicKey {
    fn verify(&self, message: &[u8], signature: &[u8]) -> bool {
        let secp = secp256k1::Secp256k1::verification_only();
        let digest = crate::utils::hash::sha256(message);
        let message = secp256k1::Message::from_digest(digest.as_array().to_owned());
        let Ok(signature) = secp256k1::ecdsa::Signature::from_compact(signature) else {
            return false;
        };
        self.0.verify(&secp, &message, &signature).is_ok()
    }
}

impl<T: AsRef<[u8]>> ToPublicKey<Secp256k1PublicKey> for T {
    fn to_public_key(&self) -> Result<Secp256k1PublicKey, ToPublicKeyError> {
        let slice = self.as_ref();
        let key = secp256k1::PublicKey::from_slice(slice)?;
        Ok(Secp256k1PublicKey(key))
    }
}

impl Secp256k1PublicKey {
    fn encode_uncompressed(&self) -> [u8; 65] {
        self.0.serialize_uncompressed()
    }

    fn encode_compressed(&self) -> [u8; 33] {
        self.0.serialize()
    }

    fn curve_point(&self) -> CurvePoint {
        let uncompressed = self.encode_uncompressed();
        let (_, xy) = uncompressed.rsplit_array_ref::<64>();
        let (x, _) = xy.split_array_ref::<32>();
        let (_, y) = xy.rsplit_array_ref::<32>();
        CurvePoint {
            x: x.to_owned(),
            y: y.to_owned(),
        }
    }
}
