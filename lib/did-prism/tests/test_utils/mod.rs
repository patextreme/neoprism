#![allow(dead_code)]

use chrono::DateTime;
use identus_apollo::crypto::secp256k1::Secp256k1PrivateKey;
use identus_did_prism::dlt::{BlockMetadata, OperationMetadata};
use identus_did_prism::proto;
use prost::Message;

const MASTER_KEY: [u8; 32] = [1; 32];

#[derive(Default)]
pub struct CreateDidOptions {
    pub contexts: Option<Vec<String>>,
    pub public_keys: Option<Vec<proto::PublicKey>>,
    pub services: Option<Vec<proto::Service>>,
}

pub fn create_did_operation(options: Option<CreateDidOptions>) -> proto::SignedPrismOperation {
    let options = options.unwrap_or_default();
    let master_sk = Secp256k1PrivateKey::from_slice(&MASTER_KEY).unwrap();
    let mut public_keys = vec![to_public_key("master-0", proto::KeyUsage::MasterKey, &master_sk)];
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
    proto::SignedPrismOperation {
        signed_with: "master-0".to_string(),
        signature: master_sk.sign(&operation.encode_to_vec()),
        operation: Some(operation),
    }
}

pub fn create_storage_operation(
    signed_with: &str,
    vdr_sk: &Secp256k1PrivateKey,
    operation: proto::ProtoCreateStorageEntry,
) -> proto::SignedPrismOperation {
    let operation_inner = proto::prism_operation::Operation::CreateStorageEntry(operation);
    let operation = proto::PrismOperation {
        operation: Some(operation_inner),
    };
    proto::SignedPrismOperation {
        signed_with: signed_with.to_string(),
        signature: vdr_sk.sign(&operation.encode_to_vec()),
        operation: Some(operation),
    }
}

pub fn to_public_key(id: &str, usage: proto::KeyUsage, sk: &Secp256k1PrivateKey) -> proto::PublicKey {
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
