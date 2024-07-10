use std::fmt::Debug;

use k256::ecdsa::signature::Verifier;
use k256::elliptic_curve::sec1::{EncodedPoint, ToEncodedPoint};
use k256::Secp256k1;

use super::{EncodeArray, EncodeVec, ToPublicKey, ToPublicKeyError, Verifiable};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Secp256k1PublicKey(k256::PublicKey);

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
        let verifying_key: k256::ecdsa::VerifyingKey = self.0.into();
        let Ok(signature) = k256::ecdsa::Signature::from_der(signature) else {
            return false;
        };
        verifying_key.verify(message, &signature).is_ok()
    }
}

impl<T: AsRef<[u8]>> ToPublicKey<Secp256k1PublicKey> for T {
    fn to_public_key(&self) -> Result<Secp256k1PublicKey, ToPublicKeyError> {
        Ok(Secp256k1PublicKey(k256::PublicKey::from_sec1_bytes(self.as_ref())?))
    }
}

impl Secp256k1PublicKey {
    fn encode_uncompressed(&self) -> [u8; 65] {
        let bytes: EncodedPoint<Secp256k1> = self.0.to_encoded_point(false);
        let bytes = bytes.to_bytes();
        let Some((chunk, _)) = bytes.split_first_chunk::<65>() else {
            unreachable!("EncodedPoint::to_bytes() must return a single chunk");
        };
        chunk.to_owned()
    }

    fn encode_compressed(&self) -> [u8; 33] {
        let bytes: EncodedPoint<Secp256k1> = self.0.to_encoded_point(true);
        let bytes = bytes.to_bytes();
        let Some((chunk, _)) = bytes.split_first_chunk::<33>() else {
            unreachable!("EncodedPoint::to_bytes() must return a single chunk");
        };
        chunk.to_owned()
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
