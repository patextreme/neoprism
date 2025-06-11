use std::ops::Deref;

use identus_apollo::crypto::secp256k1::Secp256k1PrivateKey;
use identus_apollo::hash::Sha256Digest;
use identus_did_prism::did::CanonicalPrismDid;
use identus_did_prism::did::operation::StorageData;
use identus_did_prism::proto;
use identus_did_prism::protocol::resolver;

mod test_utils;

const VDR_KEY: [u8; 32] = [2; 32];
const VDR_KEY_NAME: &str = "vdr-0";

#[test]
fn create_storage_entry() {
    let (create_did_op, _, did, vdr_sk) = create_did_with_vdr_key();
    let (create_storage_op, _) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism_operation::Operation::CreateStorageEntry(proto::ProtoCreateStorageEntry {
            did_prism_hash: did.suffix.to_vec(),
            nonce: vec![0],
            data: Some(proto::proto_create_storage_entry::Data::Bytes(vec![1, 2, 3])),
        }),
    );

    let operations = test_utils::populate_metadata(vec![create_did_op, create_storage_op]);
    let state = resolver::resolve_published(operations).0.unwrap();

    assert_eq!(state.storage.len(), 1);
    assert_eq!(*state.storage[0].data, StorageData::Bytes(vec![1, 2, 3]));
}

#[test]
fn create_multiple_storage_entries() {
    let (create_did_op, _, did, vdr_sk) = create_did_with_vdr_key();
    let (create_storage_op_1, create_storage_op_hash_1) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism_operation::Operation::CreateStorageEntry(proto::ProtoCreateStorageEntry {
            did_prism_hash: did.suffix.to_vec(),
            nonce: vec![0],
            data: Some(proto::proto_create_storage_entry::Data::Bytes(vec![1, 2, 3])),
        }),
    );
    let (create_storage_op_2, create_storage_op_hash_2) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism_operation::Operation::CreateStorageEntry(proto::ProtoCreateStorageEntry {
            did_prism_hash: did.suffix.to_vec(),
            nonce: vec![1],
            data: Some(proto::proto_create_storage_entry::Data::Bytes(vec![4, 5, 6])),
        }),
    );

    let operations = test_utils::populate_metadata(vec![create_did_op, create_storage_op_1, create_storage_op_2]);
    let state = resolver::resolve_published(operations).0.unwrap();

    assert_eq!(state.storage.len(), 2);
    assert_eq!(
        *state
            .storage
            .iter()
            .find(|s| s.init_operation_hash.deref() == &create_storage_op_hash_1)
            .unwrap()
            .data,
        StorageData::Bytes(vec![1, 2, 3])
    );
    assert_eq!(
        *state
            .storage
            .iter()
            .find(|s| s.init_operation_hash.deref() == &create_storage_op_hash_2)
            .unwrap()
            .data,
        StorageData::Bytes(vec![4, 5, 6])
    );
}

#[test]
fn update_storage_entry() {
    let (create_did_op, _, did, vdr_sk) = create_did_with_vdr_key();
    let (create_storage_op, create_storage_op_hash) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism_operation::Operation::CreateStorageEntry(proto::ProtoCreateStorageEntry {
            did_prism_hash: did.suffix.to_vec(),
            nonce: vec![0],
            data: Some(proto::proto_create_storage_entry::Data::Bytes(vec![1, 2, 3])),
        }),
    );
    let (update_storage_op, update_storage_op_hash) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism_operation::Operation::UpdateStorageEntry(proto::ProtoUpdateStorageEntry {
            previous_operation_hash: create_storage_op_hash.to_vec(),
            data: Some(proto::proto_update_storage_entry::Data::Bytes(vec![4, 5, 6])),
        }),
    );

    let operations = test_utils::populate_metadata(vec![create_did_op, create_storage_op, update_storage_op]);
    let state = resolver::resolve_published(operations).0.unwrap();

    assert_eq!(state.storage.len(), 1);
    assert_eq!(state.storage[0].init_operation_hash.deref(), &create_storage_op_hash);
    assert_eq!(state.storage[0].last_operation_hash.deref(), &update_storage_op_hash);
    assert_eq!(state.storage[0].data.deref(), &StorageData::Bytes(vec![4, 5, 6]));
}

#[test]
fn deactivate_storage_entry() {
    let (create_did_op, _, did, vdr_sk) = create_did_with_vdr_key();
    let (create_storage_op, create_storage_op_hash) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism_operation::Operation::CreateStorageEntry(proto::ProtoCreateStorageEntry {
            did_prism_hash: did.suffix.to_vec(),
            nonce: vec![0],
            data: Some(proto::proto_create_storage_entry::Data::Bytes(vec![1, 2, 3])),
        }),
    );
    let (deactivate_storage_op, _) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism_operation::Operation::DeactivateStorageEntry(proto::ProtoDeactivateStorageEntry {
            previous_operation_hash: create_storage_op_hash.to_vec(),
        }),
    );

    let operations = test_utils::populate_metadata(vec![create_did_op, create_storage_op, deactivate_storage_op]);
    let state = resolver::resolve_published(operations).0.unwrap();

    assert!(state.storage.is_empty());
}

#[test]
fn create_storage_entry_with_non_vdr_key() {
    let (create_did_op, _, did, vdr_sk) = create_did_with_vdr_key();
    let (create_storage_op, _) = test_utils::new_signed_operation(
        "master-0",
        &vdr_sk,
        proto::prism_operation::Operation::CreateStorageEntry(proto::ProtoCreateStorageEntry {
            did_prism_hash: did.suffix.to_vec(),
            nonce: vec![0],
            data: Some(proto::proto_create_storage_entry::Data::Bytes(vec![1, 2, 3])),
        }),
    );

    let operations = test_utils::populate_metadata(vec![create_did_op, create_storage_op]);
    let state = resolver::resolve_published(operations).0.unwrap();

    assert!(state.storage.is_empty());
}

#[test]
fn update_storage_entry_with_invalid_prev_operation_hash() {
    let (create_did_op, _, did, vdr_sk) = create_did_with_vdr_key();
    let (create_storage_op, _) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism_operation::Operation::CreateStorageEntry(proto::ProtoCreateStorageEntry {
            did_prism_hash: did.suffix.to_vec(),
            nonce: vec![0],
            data: Some(proto::proto_create_storage_entry::Data::Bytes(vec![1, 2, 3])),
        }),
    );
    let (update_storage_op, _) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism_operation::Operation::UpdateStorageEntry(proto::ProtoUpdateStorageEntry {
            previous_operation_hash: [0; 32].to_vec(),
            data: Some(proto::proto_update_storage_entry::Data::Bytes(vec![4, 5, 6])),
        }),
    );

    let operations = test_utils::populate_metadata(vec![create_did_op, create_storage_op, update_storage_op]);
    let state = resolver::resolve_published(operations).0.unwrap();

    assert_eq!(state.storage.len(), 1);
    assert_eq!(state.storage[0].data.deref(), &StorageData::Bytes(vec![1, 2, 3]));
}

#[test]
fn deactivate_storage_entry_with_invalid_prev_operation_hash() {
    let (create_did_op, _, did, vdr_sk) = create_did_with_vdr_key();
    let (create_storage_op, _) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism_operation::Operation::CreateStorageEntry(proto::ProtoCreateStorageEntry {
            did_prism_hash: did.suffix.to_vec(),
            nonce: vec![0],
            data: Some(proto::proto_create_storage_entry::Data::Bytes(vec![1, 2, 3])),
        }),
    );
    let (deactivate_storage_op, _) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism_operation::Operation::DeactivateStorageEntry(proto::ProtoDeactivateStorageEntry {
            previous_operation_hash: [0; 32].to_vec(),
        }),
    );

    let operations = test_utils::populate_metadata(vec![create_did_op, create_storage_op, deactivate_storage_op]);
    let state = resolver::resolve_published(operations).0.unwrap();

    assert_eq!(state.storage.len(), 1);
    assert_eq!(state.storage[0].data.deref(), &StorageData::Bytes(vec![1, 2, 3]));
}

fn create_did_with_vdr_key() -> (
    proto::SignedPrismOperation,
    Sha256Digest,
    CanonicalPrismDid,
    Secp256k1PrivateKey,
) {
    let vdr_sk = Secp256k1PrivateKey::from_slice(&VDR_KEY).unwrap();
    let options = test_utils::CreateDidOptions {
        public_keys: Some(vec![test_utils::new_public_key(
            VDR_KEY_NAME,
            proto::KeyUsage::VdrKey,
            &vdr_sk,
        )]),
        ..Default::default()
    };
    let (signed_operation, operation_hash) = test_utils::new_create_did_operation(Some(options));
    let did = CanonicalPrismDid::from_operation(signed_operation.operation.as_ref().unwrap()).unwrap();
    (signed_operation, operation_hash, did, vdr_sk)
}
