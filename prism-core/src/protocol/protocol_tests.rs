use super::{ProtocolParameter, Revocable};
use crate::{
    crypto::{ec::Secp256k1PublicKey, hash::Sha256Digest},
    did::{
        operation::{
            ParsedPublicKey, ParsedPublicKeyData, ParsedService, ParsedServiceEndpoint,
            ParsedServiceType, PublicKeyId, ServiceEndpointValue, ServiceId,
        },
        CanonicalPrismDid,
    },
    dlt::{BlockTimestamp, OperationTimestamp},
    protocol::DidStateMut,
};
use bytes::Bytes;
use chrono::{Duration, TimeZone, Utc};

fn random_master_public_key(id: &str) -> ParsedPublicKey {
    let max_id_size = ProtocolParameter::default().max_id_size;
    ParsedPublicKey {
        id: PublicKeyId::parse(id, max_id_size).unwrap(),
        data: ParsedPublicKeyData::Master {
            data: Secp256k1PublicKey::random(),
        },
    }
}

fn default_service(id: &str) -> ParsedService {
    let max_id_size = ProtocolParameter::default().max_id_size;
    ParsedService {
        id: ServiceId::parse(id, max_id_size).unwrap(),
        r#type: ParsedServiceType::Single("LinkedDomains".to_string()),
        service_endpoints: ParsedServiceEndpoint::Single(ServiceEndpointValue::URI(
            "https://example.com".to_string(),
        )),
    }
}

fn default_operation_timestamp() -> OperationTimestamp {
    OperationTimestamp {
        block_timestamp: BlockTimestamp {
            cbt: Utc.timestamp_opt(0, 0).unwrap(),
            absn: 0,
        },
        osn: 0,
    }
}

#[test]
fn revocable_is_revoked_on_non_revoked() {
    let timestamp = OperationTimestamp {
        block_timestamp: BlockTimestamp {
            cbt: Utc::now(),
            absn: 0,
        },
        osn: 0,
    };
    let revocable = Revocable::new(42, &timestamp);
    assert!(!revocable.is_revoked());
    assert!(revocable.added_at == timestamp);
}

#[test]
fn revocable_is_revoked_on_revoked() {
    let timestamp_1 = OperationTimestamp {
        block_timestamp: BlockTimestamp {
            cbt: Utc::now(),
            absn: 0,
        },
        osn: 0,
    };
    let mut timestamp_2 = timestamp_1.clone();
    timestamp_2.block_timestamp.cbt += Duration::seconds(10);
    let mut revocable = Revocable::new(42, &timestamp_1);
    revocable.revoke(&timestamp_2);
    assert!(revocable.is_revoked());
    assert!(revocable.added_at == timestamp_1);
    assert!(revocable.revoked_at == Some(timestamp_2));
}

#[test]
fn did_state_mut_with_context() {
    let did = CanonicalPrismDid::from_suffix_str(&"0".repeat(64)).unwrap();
    let mut state = DidStateMut::new(did);
    assert!(state.clone().finalize().context.is_empty());

    state.with_context(vec!["hello".to_string(), "world".to_string()]);
    assert_eq!(
        state.clone().finalize().context,
        vec!["hello".to_string(), "world".to_string()]
    );

    state.with_context(vec![]);
    assert!(state.clone().finalize().context.is_empty());
}

#[test]
fn did_state_mut_with_last_op_hash() {
    let did = CanonicalPrismDid::from_suffix_str(&"0".repeat(64)).unwrap();
    let mut state = DidStateMut::new(did);
    assert_eq!(
        state.clone().finalize().last_operation_hash.as_bytes(),
        &[0; 32]
    );

    let hash = Bytes::copy_from_slice(&[1; 32]);
    state.with_last_operation_hash(Sha256Digest::from_bytes(hash).unwrap());
    assert_eq!(
        state.clone().finalize().last_operation_hash.as_bytes(),
        &[1; 32]
    );

    let hash = Bytes::copy_from_slice(&[2; 32]);
    state.with_last_operation_hash(Sha256Digest::from_bytes(hash).unwrap());
    assert_eq!(
        state.clone().finalize().last_operation_hash.as_bytes(),
        &[2; 32]
    );
}

#[test]
fn did_state_mut_add_public_key() {
    let did = CanonicalPrismDid::from_suffix_str(&"0".repeat(64)).unwrap();
    let mut state = DidStateMut::new(did);
    assert!(state.clone().finalize().public_keys.is_empty());

    let ts = default_operation_timestamp();

    let pk_1 = random_master_public_key("master-0");
    state.add_public_key(pk_1.clone(), &ts).unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.public_keys.len(), 1);
    assert!(finalized_state.public_keys.contains(&pk_1));

    let pk_2 = random_master_public_key("master-1");
    state.add_public_key(pk_2.clone(), &ts).unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.public_keys.len(), 2);
    assert!(finalized_state.public_keys.contains(&pk_2));
}

#[test]
fn did_state_mut_add_public_key_duplicate() {
    let did = CanonicalPrismDid::from_suffix_str(&"0".repeat(64)).unwrap();
    let mut state = DidStateMut::new(did);
    assert!(state.clone().finalize().public_keys.is_empty());

    let ts = default_operation_timestamp();

    let pk_1 = random_master_public_key("master-0");
    state.add_public_key(pk_1.clone(), &ts).unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.public_keys.len(), 1);
    assert!(finalized_state.public_keys.contains(&pk_1));

    let pk_2 = random_master_public_key("master-0");
    let result = state.add_public_key(pk_2.clone(), &ts);
    assert!(result.is_err());
    assert_eq!(state.clone().finalize().public_keys.len(), 1);
}

#[test]
fn did_state_mut_add_public_key_duplicate_revoked() {
    let did = CanonicalPrismDid::from_suffix_str(&"0".repeat(64)).unwrap();
    let mut state = DidStateMut::new(did);
    assert!(state.clone().finalize().public_keys.is_empty());

    let ts = default_operation_timestamp();

    let pk_1 = random_master_public_key("master-0");
    state.add_public_key(pk_1.clone(), &ts).unwrap();
    assert_eq!(state.clone().finalize().public_keys.len(), 1);

    state.revoke_public_key(&pk_1.id, &ts).unwrap();
    assert!(state.clone().finalize().public_keys.is_empty());

    let pk_2 = random_master_public_key("master-0");
    let result = state.add_public_key(pk_2.clone(), &ts);
    assert!(result.is_err());
    assert!(state.clone().finalize().public_keys.is_empty());
}

#[test]
fn did_state_mut_revoke_public_key() {
    let did = CanonicalPrismDid::from_suffix_str(&"0".repeat(64)).unwrap();
    let mut state = DidStateMut::new(did);
    assert!(state.clone().finalize().public_keys.is_empty());

    let ts = default_operation_timestamp();

    let pk_1 = random_master_public_key("master-0");
    state.add_public_key(pk_1.clone(), &ts).unwrap();
    let finalilzed_state = state.clone().finalize();
    assert_eq!(finalilzed_state.public_keys.len(), 1);
    assert!(finalilzed_state.public_keys.contains(&pk_1));

    let pk_2 = random_master_public_key("master-1");
    state.add_public_key(pk_2.clone(), &ts).unwrap();
    let finalilzed_state = state.clone().finalize();
    assert_eq!(finalilzed_state.public_keys.len(), 2);
    assert!(finalilzed_state.public_keys.contains(&pk_2));

    state.revoke_public_key(&pk_1.id, &ts).unwrap();
    let finalilzed_state = state.clone().finalize();
    assert_eq!(finalilzed_state.public_keys.len(), 1);
    assert!(!finalilzed_state.public_keys.contains(&pk_1));
    assert!(finalilzed_state.public_keys.contains(&pk_2));
}

#[test]
fn did_state_mut_revoke_public_key_not_exist() {
    let did = CanonicalPrismDid::from_suffix_str(&"0".repeat(64)).unwrap();
    let mut state = DidStateMut::new(did);
    assert!(state.clone().finalize().public_keys.is_empty());

    let ts = default_operation_timestamp();

    let pk_1 = random_master_public_key("master-0");
    state.add_public_key(pk_1.clone(), &ts).unwrap();
    let finalilzed_state = state.clone().finalize();
    assert_eq!(finalilzed_state.public_keys.len(), 1);
    assert!(finalilzed_state.public_keys.contains(&pk_1));

    let pk_2 = random_master_public_key("master-1");
    let result = state.revoke_public_key(&pk_2.id, &ts);
    let finalilzed_state = state.clone().finalize();
    assert!(result.is_err());
    assert_eq!(finalilzed_state.public_keys.len(), 1);
}

#[test]
fn did_state_mut_revoke_public_key_already_revoked() {
    let did = CanonicalPrismDid::from_suffix_str(&"0".repeat(64)).unwrap();
    let mut state = DidStateMut::new(did);
    assert!(state.clone().finalize().public_keys.is_empty());

    let ts = default_operation_timestamp();

    let pk_1 = random_master_public_key("master-0");
    state.add_public_key(pk_1.clone(), &ts).unwrap();
    let finalilzed_state = state.clone().finalize();
    assert_eq!(finalilzed_state.public_keys.len(), 1);
    assert!(finalilzed_state.public_keys.contains(&pk_1));

    state.revoke_public_key(&pk_1.id, &ts).unwrap();
    let finalilzed_state = state.clone().finalize();
    assert_eq!(finalilzed_state.public_keys.len(), 0);

    let result = state.revoke_public_key(&pk_1.id, &ts);
    assert!(result.is_err());
    assert_eq!(finalilzed_state.public_keys.len(), 0);
}

#[test]
fn did_state_mut_add_service() {
    let did = CanonicalPrismDid::from_suffix_str(&"0".repeat(64)).unwrap();
    let mut state = DidStateMut::new(did);
    assert!(state.clone().finalize().services.is_empty());

    let ts = default_operation_timestamp();

    let service_1 = default_service("service-0");
    state.add_service(service_1.clone(), &ts).unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.services.len(), 1);
    assert!(finalized_state.services.contains(&service_1));

    let service_2 = default_service("service-1");
    state.add_service(service_2.clone(), &ts).unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.services.len(), 2);
    assert!(finalized_state.services.contains(&service_2));
}

#[test]
fn did_state_mut_add_sevice_duplicate() {
    let did = CanonicalPrismDid::from_suffix_str(&"0".repeat(64)).unwrap();
    let mut state = DidStateMut::new(did);
    assert!(state.clone().finalize().services.is_empty());

    let ts = default_operation_timestamp();

    let service_1 = default_service("service-0");
    state.add_service(service_1.clone(), &ts).unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.services.len(), 1);
    assert!(finalized_state.services.contains(&service_1));

    let service_2 = default_service("service-0");
    let result = state.add_service(service_2.clone(), &ts);
    assert!(result.is_err());
    assert_eq!(finalized_state.services.len(), 1);
}

#[test]
fn did_state_mut_add_sevice_duplicate_revoked() {
    let did = CanonicalPrismDid::from_suffix_str(&"0".repeat(64)).unwrap();
    let mut state = DidStateMut::new(did);
    assert!(state.clone().finalize().services.is_empty());

    let ts = default_operation_timestamp();

    let service_1 = default_service("service-0");
    state.add_service(service_1.clone(), &ts).unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.services.len(), 1);
    assert!(finalized_state.services.contains(&service_1));

    state.revoke_service(&service_1.id, &ts).unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.services.len(), 0);

    let service_2 = default_service("service-0");
    let result = state.add_service(service_2.clone(), &ts);
    assert!(result.is_err());
    assert_eq!(finalized_state.services.len(), 0);
}

#[test]
fn did_state_mut_revoke_service() {
    let did = CanonicalPrismDid::from_suffix_str(&"0".repeat(64)).unwrap();
    let mut state = DidStateMut::new(did);
    assert!(state.clone().finalize().services.is_empty());

    let ts = default_operation_timestamp();

    let service_1 = default_service("service-0");
    state.add_service(service_1.clone(), &ts).unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.services.len(), 1);
    assert!(finalized_state.services.contains(&service_1));

    let service_2 = default_service("service-1");
    state.add_service(service_2.clone(), &ts).unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.services.len(), 2);
    assert!(finalized_state.services.contains(&service_2));

    state.revoke_service(&service_1.id, &ts).unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.services.len(), 1);
    assert!(!finalized_state.services.contains(&service_1));
    assert!(finalized_state.services.contains(&service_2));
}

#[test]
fn did_stae_mut_revoke_service_not_exist() {
    let did = CanonicalPrismDid::from_suffix_str(&"0".repeat(64)).unwrap();
    let mut state = DidStateMut::new(did);
    assert!(state.clone().finalize().services.is_empty());

    let ts = default_operation_timestamp();

    let service_1 = default_service("service-0");
    state.add_service(service_1.clone(), &ts).unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.services.len(), 1);
    assert!(finalized_state.services.contains(&service_1));

    let service_3 = default_service("service-2");
    let result = state.revoke_service(&service_3.id, &ts);
    assert!(result.is_err());
    assert_eq!(finalized_state.services.len(), 1);
}

#[test]
fn did_state_mut_revoke_service_already_revoked() {
    let did = CanonicalPrismDid::from_suffix_str(&"0".repeat(64)).unwrap();
    let mut state = DidStateMut::new(did);
    assert!(state.clone().finalize().services.is_empty());

    let ts = default_operation_timestamp();

    let service_1 = default_service("service-0");
    state.add_service(service_1.clone(), &ts).unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.services.len(), 1);
    assert!(finalized_state.services.contains(&service_1));

    state.revoke_service(&service_1.id, &ts).unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.services.len(), 0);

    let result = state.revoke_service(&service_1.id, &ts);
    assert!(result.is_err());
    assert_eq!(finalized_state.services.len(), 0);
}

#[test]
fn did_state_mut_update_service_type() {
    let did = CanonicalPrismDid::from_suffix_str(&"0".repeat(64)).unwrap();
    let mut state = DidStateMut::new(did);
    assert!(state.clone().finalize().services.is_empty());

    let ts = default_operation_timestamp();

    let service_1 = default_service("service-0");
    state.add_service(service_1.clone(), &ts).unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.services.len(), 1);
    assert!(finalized_state.services.contains(&service_1));

    let service_type = ParsedServiceType::Single("DIDCommMessaging".to_string());
    state
        .update_service_type(&service_1.id, service_type.clone())
        .unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.services.len(), 1);
    assert_eq!(
        finalized_state.services.first().unwrap().r#type,
        service_type
    );
}

#[test]
fn did_state_mut_update_service_type_not_exist() {
    let did = CanonicalPrismDid::from_suffix_str(&"0".repeat(64)).unwrap();
    let mut state = DidStateMut::new(did);
    assert!(state.clone().finalize().services.is_empty());

    let ts = default_operation_timestamp();

    let service_1 = default_service("service-0");
    state.add_service(service_1.clone(), &ts).unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.services.len(), 1);
    assert!(finalized_state.services.contains(&service_1));

    let service_2 = default_service("service-1");
    let service_type = ParsedServiceType::Single("DIDCommMessaging".to_string());
    let result = state.update_service_type(&service_2.id, service_type.clone());
    assert!(result.is_err());
    assert_eq!(finalized_state.services.len(), 1);
    assert!(finalized_state.services.contains(&service_1));
}

#[test]
fn did_state_mut_update_service_type_already_revoked() {
    let did = CanonicalPrismDid::from_suffix_str(&"0".repeat(64)).unwrap();
    let mut state = DidStateMut::new(did);
    assert!(state.clone().finalize().services.is_empty());

    let ts = default_operation_timestamp();

    let service_1 = default_service("service-0");
    state.add_service(service_1.clone(), &ts).unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.services.len(), 1);
    assert!(finalized_state.services.contains(&service_1));

    state.revoke_service(&service_1.id, &ts).unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.services.len(), 0);

    let service_type = ParsedServiceType::Single("DIDCommMessaging".to_string());
    let result = state.update_service_type(&service_1.id, service_type.clone());
    assert!(result.is_err());
    assert_eq!(finalized_state.services.len(), 0);
}

#[test]
fn did_state_mut_update_service_endpoint() {
    let did = CanonicalPrismDid::from_suffix_str(&"0".repeat(64)).unwrap();
    let mut state = DidStateMut::new(did);
    assert!(state.clone().finalize().services.is_empty());

    let ts = default_operation_timestamp();

    let service_1 = default_service("service-0");
    state.add_service(service_1.clone(), &ts).unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.services.len(), 1);
    assert!(finalized_state.services.contains(&service_1));

    let endpoint = ParsedServiceEndpoint::Single(ServiceEndpointValue::URI(
        "https://example.com/endpoint".to_string(),
    ));
    state
        .update_service_endpoint(&service_1.id, endpoint.clone())
        .unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.services.len(), 1);
    assert_eq!(
        finalized_state.services.first().unwrap().service_endpoints,
        endpoint
    );
}

#[test]
fn did_state_mut_update_service_endpoint_not_exist() {
    let did = CanonicalPrismDid::from_suffix_str(&"0".repeat(64)).unwrap();
    let mut state = DidStateMut::new(did);
    assert!(state.clone().finalize().services.is_empty());

    let ts = default_operation_timestamp();

    let service_1 = default_service("service-0");
    state.add_service(service_1.clone(), &ts).unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.services.len(), 1);
    assert!(finalized_state.services.contains(&service_1));

    let service_2 = default_service("service-1");
    let endpoint = ParsedServiceEndpoint::Single(ServiceEndpointValue::URI(
        "https://example.com/endpoint".to_string(),
    ));
    let result = state.update_service_endpoint(&service_2.id, endpoint.clone());
    assert!(result.is_err());
    assert_eq!(finalized_state.services.len(), 1);
    assert!(finalized_state.services.contains(&service_1));
}

#[test]
fn did_state_mut_update_service_already_revoked() {
    let did = CanonicalPrismDid::from_suffix_str(&"0".repeat(64)).unwrap();
    let mut state = DidStateMut::new(did);
    assert!(state.clone().finalize().services.is_empty());

    let ts = default_operation_timestamp();

    let service_1 = default_service("service-0");
    state.add_service(service_1.clone(), &ts).unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.services.len(), 1);
    assert!(finalized_state.services.contains(&service_1));

    state.revoke_service(&service_1.id, &ts).unwrap();
    let finalized_state = state.clone().finalize();
    assert_eq!(finalized_state.services.len(), 0);

    let endpoint = ParsedServiceEndpoint::Single(ServiceEndpointValue::URI(
        "https://example.com/endpoint".to_string(),
    ));
    let result = state.update_service_endpoint(&service_1.id, endpoint.clone());
    assert!(result.is_err());
    assert_eq!(finalized_state.services.len(), 0);
}
