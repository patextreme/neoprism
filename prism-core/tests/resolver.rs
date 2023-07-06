use bytes::Bytes;
use chrono::{Duration, Utc};
use prism_core::{
    crypto::codec::{Base64UrlStrNoPad, HexStr},
    proto::{
        create_did_operation::DidCreationData, public_key::KeyData, KeyUsage, PublicKey,
        SignedAtalaOperation,
    },
    protocol::resolver::resolve,
};
use prost::Message;
use secp256k1::SecretKey;
use std::str::FromStr;

mod common;

fn valid_signed_create_operation() -> SignedAtalaOperation {
    let (public_key, private_key) = common::crypto::default_secp256k1_keypair();
    valid_signed_create_operation_with_keypair(public_key, private_key)
}

fn valid_signed_create_operation_with_keypair(
    public_key: KeyData,
    private_key: SecretKey,
) -> SignedAtalaOperation {
    let did_data = DidCreationData {
        public_keys: vec![PublicKey {
            id: "master-0".to_string(),
            usage: KeyUsage::MasterKey.into(),
            key_data: Some(public_key),
        }],
        ..Default::default()
    };
    let operation = common::operation::create_did_operation(did_data);
    common::operation::sign_operation("master-0", &private_key, &operation)
}

#[test]
fn resolve_no_operation() {
    let result = resolve(vec![]);
    assert!(result.is_err());
}

#[test]
fn resolve_valid_create_operation() {
    let signed_operation = valid_signed_create_operation();
    let metadata = common::time::default_operation_metadata();
    let result = resolve(vec![(metadata, signed_operation)]).unwrap();

    assert_eq!(
        result.did.to_string(),
        "did:prism:41a7b49a8a2007595970a1b9ec532f04c98b32414d2e0ef8df6fa033e77be6e3"
    );
    assert_eq!(result.did.suffix, result.last_operation_hash);
    assert_eq!(result.public_keys.len(), 1);
    assert!(result.services.is_empty());
    assert!(result.context.is_empty());
}

#[test]
fn resolve_update_operation_only() {
    let (_, private_key) = common::crypto::default_secp256k1_keypair();
    let operation = common::operation::update_did_operation(
        "41a7b49a8a2007595970a1b9ec532f04c98b32414d2e0ef8df6fa033e77be6e3",
        HexStr::from_str("41a7b49a8a2007595970a1b9ec532f04c98b32414d2e0ef8df6fa033e77be6e3")
            .unwrap()
            .as_bytes()
            .into(),
        vec![],
    );
    let signed_operation = common::operation::sign_operation("master-0", &private_key, &operation);
    let metadata = common::time::default_operation_metadata();
    let result = resolve(vec![(metadata, signed_operation)]);
    assert!(result.is_err());
}

#[test]
fn resolve_empty_signed_operation() {
    let signed_operation = SignedAtalaOperation::default();
    let metadata = common::time::default_operation_metadata();
    let result = resolve(vec![(metadata, signed_operation)]);
    assert!(result.is_err());
}

#[test]
fn resolve_create_operation_incorrect_signature() {
    let mut signed_operation = valid_signed_create_operation();
    signed_operation.signature = vec![];

    let metadata = common::time::default_operation_metadata();
    let result = resolve(vec![(metadata, signed_operation)]);
    assert!(result.is_err());
}

#[test]
fn resolve_create_operation_incorrect_signed_with() {
    let mut signed_operation = valid_signed_create_operation();
    signed_operation.signed_with = "hello".to_string();

    let metadata = common::time::default_operation_metadata();
    let result = resolve(vec![(metadata, signed_operation)]);
    assert!(result.is_err());
}

#[test]
fn resolve_create_operation_without_master_key() {
    let (public_key, private_key) = common::crypto::default_secp256k1_keypair();
    let did_data = DidCreationData {
        public_keys: vec![PublicKey {
            id: "issuing-0".to_string(),
            usage: KeyUsage::IssuingKey.into(),
            key_data: Some(public_key),
        }],
        ..Default::default()
    };
    let operation = common::operation::create_did_operation(did_data);
    let signed_operation = common::operation::sign_operation("master-0", &private_key, &operation);

    let metadata = common::time::default_operation_metadata();
    let result = resolve(vec![(metadata, signed_operation)]);
    assert!(result.is_err());
}

#[test]
fn resolve_multiple_create_operations_use_first() {
    let signed_operation_1 = valid_signed_create_operation();

    let (pk, sk) = common::crypto::random_secp256k1_keypair();
    let signed_operation_2 = valid_signed_create_operation_with_keypair(pk, sk);

    let now = Utc::now();
    let metadata_1 = common::time::operation_metadata(&now);
    let metadata_2 = common::time::operation_metadata(&(now + Duration::seconds(1)));

    let result = resolve(vec![
        (metadata_2, signed_operation_2),
        (metadata_1, signed_operation_1),
    ])
    .unwrap();
    assert_eq!(
        result.did.to_string(),
        "did:prism:41a7b49a8a2007595970a1b9ec532f04c98b32414d2e0ef8df6fa033e77be6e3"
    );
}

#[test]
fn resolve_operation_from_other_implementation() {
    let bytes: Bytes = Base64UrlStrNoPad::from_str("CgdtYXN0ZXIwEkYwRAIgUMO1cMOGnrJo5gBsglf4IgjXwl8sI-kADOchpLJNEekCIC4XCdAUZ1okPhtxaJhLk7VLY-qIiHApwQ5_08ry8mEtGsABCr0BCroBEloKBWhlbGxvEARCTwoJc2VjcDI1NmsxEiDyW4X0Dox1DUjgfM1PEVkfnR20YlhTQWF_hTdCP2I9hRogJFqEGMCk2lXUKxptpNOKMegFLehtCob0MlFyAuUaJXYSXAoHbWFzdGVyMBABQk8KCXNlY3AyNTZrMRIgDZq1LcthL4TnncuNvB09MlX8JwtVNif5kov7S0UTI5EaIEQ8sZhQ0CyMHDF9cT74BEO5CTCDpgrF5H67A-QsPxXi").unwrap().into();
    let signed_operation = SignedAtalaOperation::decode(bytes).unwrap();
    let metadata = common::time::default_operation_metadata();
    let result = resolve(vec![(metadata, signed_operation)]).unwrap();
    assert_eq!(
        result.did.to_string(),
        "did:prism:1bb9001c56e1090438bc89756ff93d7c6ff6848d5f8bf20b6568a96449c2f38d"
    );
}
