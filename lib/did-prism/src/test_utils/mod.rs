use chrono::DateTime;
use identus_apollo::crypto::secp256k1::Secp256k1PrivateKey;
use identus_apollo::hash::Sha256Digest;
use prost::Message;

use crate::dlt::{BlockMetadata, OperationMetadata};
use crate::proto;

const MASTER_KEY: [u8; 32] = [1; 32];
const MASTER_KEY_NAME: &str = "master-0";

#[derive(Default)]
pub struct CreateDidOptions {
    pub contexts: Option<Vec<String>>,
    pub public_keys: Option<Vec<proto::PublicKey>>,
    pub services: Option<Vec<proto::Service>>,
}

pub fn new_create_did_operation(
    options: Option<CreateDidOptions>,
) -> (proto::SignedPrismOperation, Sha256Digest, Secp256k1PrivateKey) {
    let options = options.unwrap_or_default();
    let master_sk = Secp256k1PrivateKey::from_slice(&MASTER_KEY).unwrap();
    let mut public_keys = vec![new_public_key(MASTER_KEY_NAME, proto::KeyUsage::MasterKey, &master_sk)];
    public_keys.extend_from_slice(&options.public_keys.unwrap_or_default());
    let operation_inner = proto::prism_operation::Operation::CreateDid(proto::ProtoCreateDid {
        did_data: Some(proto::proto_create_did::DidCreationData {
            public_keys,
            services: options.services.unwrap_or_default(),
            context: options.contexts.unwrap_or_default(),
        }),
    });
    let operation = proto::PrismOperation {
        operation: Some(operation_inner),
    };
    let operation_hash = operation.operation_hash();
    let signed_operation = proto::SignedPrismOperation {
        signed_with: MASTER_KEY_NAME.to_string(),
        signature: master_sk.sign(&operation.encode_to_vec()),
        operation: Some(operation),
    };
    (signed_operation, operation_hash, master_sk)
}

pub fn new_signed_operation(
    signed_with: &str,
    signing_key: &Secp256k1PrivateKey,
    operation: proto::prism_operation::Operation,
) -> (proto::SignedPrismOperation, Sha256Digest) {
    let operation = proto::PrismOperation {
        operation: Some(operation),
    };
    let operation_hash = operation.operation_hash();
    let signed_operation = proto::SignedPrismOperation {
        signed_with: signed_with.to_string(),
        signature: signing_key.sign(&operation.encode_to_vec()),
        operation: Some(operation),
    };
    (signed_operation, operation_hash)
}

pub fn new_public_key(id: &str, usage: proto::KeyUsage, sk: &Secp256k1PrivateKey) -> proto::PublicKey {
    let pk = sk.to_public_key();
    proto::PublicKey {
        id: id.to_string(),
        usage: usage.into(),
        key_data: Some(proto::public_key::KeyData::CompressedEcKeyData(
            proto::CompressedEcKeyData {
                curve: "secp256k1".to_string(),
                data: pk.encode_compressed().into(),
            },
        )),
    }
}

pub fn populate_metadata(
    operations: Vec<proto::SignedPrismOperation>,
) -> Vec<(OperationMetadata, proto::SignedPrismOperation)> {
    let dummy_metadata = OperationMetadata {
        block_metadata: BlockMetadata {
            slot_number: 0,
            block_number: 0,
            cbt: DateTime::UNIX_EPOCH,
            absn: 0,
        },
        osn: 0,
    };
    operations
        .into_iter()
        .enumerate()
        .map(|(idx, op)| {
            let metadata = OperationMetadata {
                osn: idx as u32,
                ..dummy_metadata.clone()
            };
            (metadata, op)
        })
        .collect()
}
