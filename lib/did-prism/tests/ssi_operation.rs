use identus_apollo::crypto::secp256k1::Secp256k1PrivateKey;
use identus_did_prism::did::operation::KeyUsage;
use identus_did_prism::proto;
use identus_did_prism::protocol::resolver;

mod utils;

#[test]
fn create_did_only_master_key() {
    let create_did_operation = utils::create_did_operation(None);
    let operations = utils::populate_metadata(vec![create_did_operation]);

    let state = resolver::resolve_published(operations).0.unwrap();

    let master_key = state
        .public_keys
        .iter()
        .find(|pk| pk.id.as_str() == "master-0")
        .unwrap();

    assert_eq!(state.services.len(), 0);
    assert_eq!(state.context.len(), 0);
    assert_eq!(state.storage.len(), 0);
    assert_eq!(state.public_keys.len(), 1);
    assert_eq!(master_key.data.usage(), KeyUsage::MasterKey);
}

#[test]
fn create_did_with_non_master_key() {
    let vdr_sk = Secp256k1PrivateKey::from_slice(&[2; 32]).unwrap();
    let auth_sk = Secp256k1PrivateKey::from_slice(&[3; 32]).unwrap();

    let options = utils::CreateDidOptions {
        public_keys: Some(vec![
            utils::to_public_key("vdr-0", proto::KeyUsage::VdrKey, &vdr_sk),
            utils::to_public_key("auth-0", proto::KeyUsage::AuthenticationKey, &auth_sk),
        ]),
        ..Default::default()
    };
    let create_did_operation = utils::create_did_operation(Some(options));
    let operations = utils::populate_metadata(vec![create_did_operation]);

    let state = resolver::resolve_published(operations).0.unwrap();

    let vdr_key = state.public_keys.iter().find(|pk| pk.id.as_str() == "vdr-0").unwrap();
    let auth_key = state.public_keys.iter().find(|pk| pk.id.as_str() == "auth-0").unwrap();
    let master_key = state
        .public_keys
        .iter()
        .find(|pk| pk.id.as_str() == "master-0")
        .unwrap();

    assert_eq!(state.public_keys.len(), 3);
    assert_eq!(master_key.data.usage(), KeyUsage::MasterKey);
    assert_eq!(vdr_key.data.usage(), KeyUsage::VdrKey);
    assert_eq!(auth_key.data.usage(), KeyUsage::AuthenticationKey);
}
