use std::ops::Deref;

use identus_apollo::crypto::secp256k1::Secp256k1PrivateKey;
use identus_apollo::hash::Sha256Digest;
use identus_did_prism::did::operation::StorageData;
use identus_did_prism::did::{CanonicalPrismDid, PrismDidOps};
use identus_did_prism::proto;
use identus_did_prism::protocol::resolver;

const VDR_KEY: [u8; 32] = [2; 32];
const VDR_KEY_NAME: &str = "vdr-0";

mod test_utils;

#[test]
fn create_storage_entry() {
    let (create_did_op, _, did, _, vdr_sk) = create_did_with_vdr_key();
    let (create_storage_op, _) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism::prism_operation::Operation::CreateStorageEntry(proto::prism_storage::ProtoCreateStorageEntry {
            did_prism_hash: did.suffix.to_vec(),
            nonce: vec![0],
            data: Some(proto::prism_storage::proto_create_storage_entry::Data::Bytes(vec![
                1, 2, 3,
            ])),
            special_fields: Default::default(),
        }),
    );

    let operations = test_utils::populate_metadata(vec![create_did_op, create_storage_op]);
    let state = resolver::resolve_published(operations).0.unwrap();

    assert_eq!(state.storage.len(), 1);
    assert_eq!(*state.storage[0].data, StorageData::Bytes(vec![1, 2, 3]));
}

#[test]
fn create_multiple_storage_entries() {
    let (create_did_op, _, did, _, vdr_sk) = create_did_with_vdr_key();
    let (create_storage_op_1, create_storage_op_hash_1) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism::prism_operation::Operation::CreateStorageEntry(proto::prism_storage::ProtoCreateStorageEntry {
            did_prism_hash: did.suffix.to_vec(),
            nonce: vec![0],
            data: Some(proto::prism_storage::proto_create_storage_entry::Data::Bytes(vec![
                1, 2, 3,
            ])),
            special_fields: Default::default(),
        }),
    );
    let (create_storage_op_2, create_storage_op_hash_2) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism::prism_operation::Operation::CreateStorageEntry(proto::prism_storage::ProtoCreateStorageEntry {
            did_prism_hash: did.suffix.to_vec(),
            nonce: vec![1],
            data: Some(proto::prism_storage::proto_create_storage_entry::Data::Bytes(vec![
                4, 5, 6,
            ])),
            special_fields: Default::default(),
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
    let (create_did_op, _, did, _, vdr_sk) = create_did_with_vdr_key();
    let (create_storage_op, create_storage_op_hash) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism::prism_operation::Operation::CreateStorageEntry(proto::prism_storage::ProtoCreateStorageEntry {
            did_prism_hash: did.suffix.to_vec(),
            nonce: vec![0],
            data: Some(proto::prism_storage::proto_create_storage_entry::Data::Bytes(vec![
                1, 2, 3,
            ])),
            special_fields: Default::default(),
        }),
    );
    let (update_storage_op, update_storage_op_hash) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism::prism_operation::Operation::UpdateStorageEntry(proto::prism_storage::ProtoUpdateStorageEntry {
            previous_event_hash: create_storage_op_hash.to_vec(),
            data: Some(proto::prism_storage::proto_update_storage_entry::Data::Bytes(vec![
                4, 5, 6,
            ])),
            special_fields: Default::default(),
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
    let (create_did_op, _, did, _, vdr_sk) = create_did_with_vdr_key();
    let (create_storage_op, create_storage_op_hash) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism::prism_operation::Operation::CreateStorageEntry(proto::prism_storage::ProtoCreateStorageEntry {
            did_prism_hash: did.suffix.to_vec(),
            nonce: vec![0],
            data: Some(proto::prism_storage::proto_create_storage_entry::Data::Bytes(vec![
                1, 2, 3,
            ])),
            special_fields: Default::default(),
        }),
    );
    let (deactivate_storage_op, _) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism::prism_operation::Operation::DeactivateStorageEntry(
            proto::prism_storage::ProtoDeactivateStorageEntry {
                previous_event_hash: create_storage_op_hash.to_vec(),
                special_fields: Default::default(),
            },
        ),
    );

    let operations = test_utils::populate_metadata(vec![create_did_op, create_storage_op, deactivate_storage_op]);
    let state = resolver::resolve_published(operations).0.unwrap();

    assert!(state.storage.is_empty());
}

#[test]
fn create_storage_entry_with_non_vdr_key() {
    let (create_did_op, _, did, _, vdr_sk) = create_did_with_vdr_key();
    let (create_storage_op, _) = test_utils::new_signed_operation(
        "master-0",
        &vdr_sk,
        proto::prism::prism_operation::Operation::CreateStorageEntry(proto::prism_storage::ProtoCreateStorageEntry {
            did_prism_hash: did.suffix.to_vec(),
            nonce: vec![0],
            data: Some(proto::prism_storage::proto_create_storage_entry::Data::Bytes(vec![
                1, 2, 3,
            ])),
            special_fields: Default::default(),
        }),
    );

    let operations = test_utils::populate_metadata(vec![create_did_op, create_storage_op]);
    let state = resolver::resolve_published(operations).0.unwrap();

    assert!(state.storage.is_empty());
}

#[test]
fn update_storage_entry_with_invalid_prev_event_hash() {
    let (create_did_op, _, did, _, vdr_sk) = create_did_with_vdr_key();
    let (create_storage_op, _) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism::prism_operation::Operation::CreateStorageEntry(proto::prism_storage::ProtoCreateStorageEntry {
            did_prism_hash: did.suffix.to_vec(),
            nonce: vec![0],
            data: Some(proto::prism_storage::proto_create_storage_entry::Data::Bytes(vec![
                1, 2, 3,
            ])),
            special_fields: Default::default(),
        }),
    );
    let (update_storage_op_1, update_op_hash_1) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism::prism_operation::Operation::UpdateStorageEntry(proto::prism_storage::ProtoUpdateStorageEntry {
            previous_event_hash: [0; 32].to_vec(),
            data: Some(proto::prism_storage::proto_update_storage_entry::Data::Bytes(vec![
                4, 5, 6,
            ])),
            special_fields: Default::default(),
        }),
    );
    let (update_storage_op_2, _) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism::prism_operation::Operation::UpdateStorageEntry(proto::prism_storage::ProtoUpdateStorageEntry {
            previous_event_hash: update_op_hash_1.to_vec(),
            data: Some(proto::prism_storage::proto_update_storage_entry::Data::Bytes(vec![
                4, 5, 6,
            ])),
            special_fields: Default::default(),
        }),
    );

    let operations = test_utils::populate_metadata(vec![
        create_did_op,
        create_storage_op,
        update_storage_op_1,
        update_storage_op_2,
    ]);
    let state = resolver::resolve_published(operations).0.unwrap();

    assert_eq!(state.storage.len(), 1);
    assert_eq!(state.storage[0].data.deref(), &StorageData::Bytes(vec![1, 2, 3]));
}

#[test]
fn update_storage_entry_with_non_vdr_key() {
    let (create_did_op, _, did, master_sk, vdr_sk) = create_did_with_vdr_key();
    let (create_storage_op, create_storage_op_hash) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism::prism_operation::Operation::CreateStorageEntry(proto::prism_storage::ProtoCreateStorageEntry {
            did_prism_hash: did.suffix.to_vec(),
            nonce: vec![0],
            data: Some(proto::prism_storage::proto_create_storage_entry::Data::Bytes(vec![
                1, 2, 3,
            ])),
            special_fields: Default::default(),
        }),
    );
    let (update_storage_op, _) = test_utils::new_signed_operation(
        "master-0",
        &master_sk,
        proto::prism::prism_operation::Operation::UpdateStorageEntry(proto::prism_storage::ProtoUpdateStorageEntry {
            previous_event_hash: create_storage_op_hash.to_vec(),
            data: Some(proto::prism_storage::proto_update_storage_entry::Data::Bytes(vec![
                4, 5, 6,
            ])),
            special_fields: Default::default(),
        }),
    );

    let operations = test_utils::populate_metadata(vec![create_did_op, create_storage_op, update_storage_op]);
    let state = resolver::resolve_published(operations).0.unwrap();

    assert_eq!(state.storage.len(), 1);
    assert_eq!(state.storage[0].data.deref(), &StorageData::Bytes(vec![1, 2, 3]));
}

#[test]
fn update_storage_entry_with_revoked_key() {
    let (create_did_op, _, did, master_sk, vdr_sk) = create_did_with_vdr_key();
    let (create_storage_op, create_storage_op_hash) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism::prism_operation::Operation::CreateStorageEntry(proto::prism_storage::ProtoCreateStorageEntry {
            did_prism_hash: did.suffix.to_vec(),
            nonce: vec![0],
            data: Some(proto::prism_storage::proto_create_storage_entry::Data::Bytes(vec![
                1, 2, 3,
            ])),
            special_fields: Default::default(),
        }),
    );
    let (revoke_key_op, revoke_key_op_hash) = test_utils::new_signed_operation(
        "master-0",
        &master_sk,
        proto::prism::prism_operation::Operation::UpdateDid(proto::prism_ssi::ProtoUpdateDID {
            previous_operation_hash: create_storage_op_hash.to_vec(),
            id: did.suffix_hex().to_string(),
            actions: vec![proto::prism_ssi::UpdateDIDAction {
                action: Some(proto::prism_ssi::update_didaction::Action::RemoveKey(
                    proto::prism_ssi::RemoveKeyAction {
                        keyId: VDR_KEY_NAME.to_string(),
                        special_fields: Default::default(),
                    },
                )),
                special_fields: Default::default(),
            }],
            special_fields: Default::default(),
        }),
    );
    let (update_storage_op, _) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism::prism_operation::Operation::UpdateStorageEntry(proto::prism_storage::ProtoUpdateStorageEntry {
            previous_event_hash: revoke_key_op_hash.to_vec(),
            data: Some(proto::prism_storage::proto_update_storage_entry::Data::Bytes(vec![
                4, 5, 6,
            ])),
            special_fields: Default::default(),
        }),
    );

    let operations =
        test_utils::populate_metadata(vec![create_did_op, create_storage_op, revoke_key_op, update_storage_op]);
    let state = resolver::resolve_published(operations).0.unwrap();

    assert_eq!(state.storage.len(), 1);
    assert_eq!(state.storage[0].data.deref(), &StorageData::Bytes(vec![1, 2, 3]));
}

#[test]
fn create_storage_entry_with_revoked_key() {
    let (create_did_op, create_did_op_hash, did, master_sk, vdr_sk) = create_did_with_vdr_key();
    let (revoke_key_op, _) = test_utils::new_signed_operation(
        "master-0",
        &master_sk,
        proto::prism::prism_operation::Operation::UpdateDid(proto::prism_ssi::ProtoUpdateDID {
            previous_operation_hash: create_did_op_hash.to_vec(),
            id: did.suffix_hex().to_string(),
            actions: vec![proto::prism_ssi::UpdateDIDAction {
                action: Some(proto::prism_ssi::update_didaction::Action::RemoveKey(
                    proto::prism_ssi::RemoveKeyAction {
                        keyId: VDR_KEY_NAME.to_string(),
                        special_fields: Default::default(),
                    },
                )),
                special_fields: Default::default(),
            }],
            special_fields: Default::default(),
        }),
    );
    let (create_storage_op, _) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism::prism_operation::Operation::CreateStorageEntry(proto::prism_storage::ProtoCreateStorageEntry {
            did_prism_hash: did.suffix.to_vec(),
            nonce: vec![0],
            data: Some(proto::prism_storage::proto_create_storage_entry::Data::Bytes(vec![
                1, 2, 3,
            ])),
            special_fields: Default::default(),
        }),
    );

    let operations = test_utils::populate_metadata(vec![create_did_op, revoke_key_op, create_storage_op]);
    let state = resolver::resolve_published(operations).0.unwrap();

    assert_eq!(state.storage.len(), 0);
}

#[test]
fn deactivate_storage_entry_with_invalid_prev_operation_hash() {
    let (create_did_op, _, did, _, vdr_sk) = create_did_with_vdr_key();
    let (create_storage_op, _) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism::prism_operation::Operation::CreateStorageEntry(proto::prism_storage::ProtoCreateStorageEntry {
            did_prism_hash: did.suffix.to_vec(),
            nonce: vec![0],
            data: Some(proto::prism_storage::proto_create_storage_entry::Data::Bytes(vec![
                1, 2, 3,
            ])),
            special_fields: Default::default(),
        }),
    );
    let (deactivate_storage_op, _) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism::prism_operation::Operation::DeactivateStorageEntry(
            proto::prism_storage::ProtoDeactivateStorageEntry {
                previous_event_hash: [0; 32].to_vec(),
                special_fields: Default::default(),
            },
        ),
    );

    let operations = test_utils::populate_metadata(vec![create_did_op, create_storage_op, deactivate_storage_op]);
    let state = resolver::resolve_published(operations).0.unwrap();

    assert_eq!(state.storage.len(), 1);
    assert_eq!(state.storage[0].data.deref(), &StorageData::Bytes(vec![1, 2, 3]));
}

#[test]
fn storage_revoked_after_deactivate_did() {
    let (create_did_op, _, did, master_sk, vdr_sk) = create_did_with_vdr_key();
    let (create_storage_op, create_storage_op_hash) = test_utils::new_signed_operation(
        VDR_KEY_NAME,
        &vdr_sk,
        proto::prism::prism_operation::Operation::CreateStorageEntry(proto::prism_storage::ProtoCreateStorageEntry {
            did_prism_hash: did.suffix.to_vec(),
            nonce: vec![0],
            data: Some(proto::prism_storage::proto_create_storage_entry::Data::Bytes(vec![
                1, 2, 3,
            ])),
            special_fields: Default::default(),
        }),
    );
    let (deactivate_did_op, _) = test_utils::new_signed_operation(
        "master-0",
        &master_sk,
        proto::prism::prism_operation::Operation::DeactivateDid(proto::prism_ssi::ProtoDeactivateDID {
            previous_operation_hash: create_storage_op_hash.to_vec(),
            id: did.suffix_hex().to_string(),
            special_fields: Default::default(),
        }),
    );

    let operations = test_utils::populate_metadata(vec![create_did_op, create_storage_op, deactivate_did_op]);
    let state = resolver::resolve_published(operations).0.unwrap();

    assert!(state.storage.is_empty());
}

fn create_did_with_vdr_key() -> (
    proto::prism::SignedPrismOperation,
    Sha256Digest,
    CanonicalPrismDid,
    Secp256k1PrivateKey,
    Secp256k1PrivateKey,
) {
    let vdr_sk = Secp256k1PrivateKey::from_slice(&VDR_KEY).unwrap();
    let options = test_utils::CreateDidOptions {
        public_keys: Some(vec![test_utils::new_public_key(
            VDR_KEY_NAME,
            proto::prism_ssi::KeyUsage::VDR_KEY,
            &vdr_sk,
        )]),
        ..Default::default()
    };
    let (signed_operation, operation_hash, master_sk) = test_utils::new_create_did_operation(Some(options));
    let did = CanonicalPrismDid::from_operation(signed_operation.operation.as_ref().unwrap()).unwrap();
    (signed_operation, operation_hash, did, master_sk, vdr_sk)
}
