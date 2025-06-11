use identus_apollo::crypto::secp256k1::Secp256k1PrivateKey;
use identus_did_prism::did::CanonicalPrismDid;
use identus_did_prism::did::operation::StorageData;
use identus_did_prism::proto;
use identus_did_prism::protocol::resolver;

mod test_utils;

const VDR_KEY: [u8; 32] = [2; 32];
const VDR_KEY_NAME: &str = "vdr-0";

#[test]
fn create_storage_entry() {
    let (create_did_operation, did, vdr_sk) = create_did_with_vdr_key();
    let create_storage_operation = test_utils::create_storage_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::ProtoCreateStorageEntry {
            did_prism_hash: did.suffix.to_vec(),
            nonce: vec![0],
            data: Some(proto::proto_create_storage_entry::Data::Bytes(vec![1, 2, 3])),
        },
    );

    let operations = test_utils::populate_metadata(vec![create_did_operation, create_storage_operation]);
    let state = resolver::resolve_published(operations).0.unwrap();

    assert_eq!(state.public_keys.len(), 2);
    assert_eq!(state.storage.len(), 1);
    assert_eq!(*state.storage[0].data, StorageData::Bytes(vec![1, 2, 3]));
    assert_eq!(
        state.storage[0].init_operation_hash,
        state.storage[0].prev_operation_hash
    );
}

fn create_did_with_vdr_key() -> (proto::SignedPrismOperation, CanonicalPrismDid, Secp256k1PrivateKey) {
    let vdr_sk = Secp256k1PrivateKey::from_slice(&VDR_KEY).unwrap();
    let options = test_utils::CreateDidOptions {
        public_keys: Some(vec![test_utils::to_public_key(
            VDR_KEY_NAME,
            proto::KeyUsage::VdrKey,
            &vdr_sk,
        )]),
        ..Default::default()
    };
    let create_operation = test_utils::create_did_operation(Some(options));
    let did = CanonicalPrismDid::from_operation(create_operation.operation.as_ref().unwrap()).unwrap();
    (create_operation, did, vdr_sk)
}
