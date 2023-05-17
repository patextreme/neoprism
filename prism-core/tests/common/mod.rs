pub mod time {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use prism_core::dlt::{BlockTimestamp, OperationTimestamp};

    pub fn default_operation_timestamp() -> OperationTimestamp {
        operation_timestamp(&DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
            Utc,
        ))
    }

    pub fn operation_timestamp(datetime: &DateTime<Utc>) -> OperationTimestamp {
        OperationTimestamp {
            block_timestamp: BlockTimestamp {
                cbt: datetime.clone(),
                absn: 0,
            },
            osn: 0,
        }
    }
}

pub mod crypto {
    use prism_core::proto::{public_key::KeyData, CompressedEcKeyData};
    use rand::rngs::OsRng;
    use secp256k1::{PublicKey, Secp256k1, SecretKey};

    pub fn default_secp256k1_keypair() -> (KeyData, SecretKey) {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&[0x42; 32]).unwrap();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let key_data = KeyData::CompressedEcKeyData(CompressedEcKeyData {
            curve: "secp256k1".to_string(),
            data: public_key.serialize().to_vec(),
        });
        (key_data, secret_key)
    }

    pub fn random_secp256k1_keypair() -> (KeyData, SecretKey) {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
        let key_data = KeyData::CompressedEcKeyData(CompressedEcKeyData {
            curve: "secp256k1".to_string(),
            data: public_key.serialize().to_vec(),
        });
        (key_data, secret_key)
    }
}

pub mod operation {
    use prism_core::{
        crypto::hash::sha256,
        proto::{
            atala_operation::Operation, create_did_operation::DidCreationData, AtalaOperation,
            CreateDidOperation, SignedAtalaOperation, UpdateDidAction, UpdateDidOperation,
        },
        util::MessageExt,
    };
    use secp256k1::{Secp256k1, SecretKey};

    pub fn sign_operation(
        signed_with: &str,
        secret_key: &SecretKey,
        operation: &AtalaOperation,
    ) -> SignedAtalaOperation {
        let secp = Secp256k1::new();
        let message = operation.encode_to_bytes().unwrap();
        let msg = secp256k1::Message::from_slice(sha256(message).as_bytes()).unwrap();
        let signature = &*secp.sign_ecdsa(&msg, secret_key).serialize_der();
        SignedAtalaOperation {
            signed_with: signed_with.to_string(),
            signature: signature.into(),
            operation: Some(operation.clone()),
        }
    }

    pub fn create_did_operation(did_data: DidCreationData) -> AtalaOperation {
        let create_operation = CreateDidOperation {
            did_data: Some(did_data),
        };
        AtalaOperation {
            operation: Some(Operation::CreateDid(create_operation)),
        }
    }

    pub fn update_did_operation(
        id: &str,
        previous_operation_hash: &[u8],
        actions: Vec<UpdateDidAction>,
    ) -> AtalaOperation {
        let update_operation = UpdateDidOperation {
            previous_operation_hash: previous_operation_hash.into(),
            id: id.to_string(),
            actions,
        };

        AtalaOperation {
            operation: Some(Operation::UpdateDid(update_operation)),
        }
    }
}
