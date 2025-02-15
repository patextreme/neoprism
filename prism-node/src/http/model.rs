pub mod form {
    use rocket::FromForm;

    #[derive(Debug, Clone, FromForm)]
    pub struct HxRpcForm {
        pub rpc: String,
    }
}

pub mod hx {
    use rocket::serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(crate = "rocket::serde")]
    pub enum HxRpc {
        GetExplorerDltCursor {},
        GetExplorerDidList { page: Option<u64> },
    }
}

pub mod api {
    use prism_core::crypto;
    use prism_core::did::operation::KeyUsage;
    use prism_core::did::{operation, DidState};
    use rocket::serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(crate = "rocket::serde", rename_all = "camelCase")]
    pub struct DidDocument {
        #[serde(rename(serialize = "@context", deserialize = "@context"))]
        context: Vec<String>,
        id: String,
        verification_method: Vec<serde_json::Value>,
        authentication: Option<Vec<String>>,
        assertion_method: Option<Vec<String>>,
        key_agreement: Option<Vec<String>>,
        capability_invocation: Option<Vec<String>>,
        capability_delegation: Option<Vec<String>>,
    }

    impl DidDocument {
        pub fn new(did: &str, did_state: DidState) -> Self {
            let mut context = vec!["https://www.w3.org/ns/did/v1".to_string()];
            context.extend(did_state.context);

            let to_jwk_json = |k: &operation::PublicKey| -> Option<serde_json::Value> {
                match &k.data {
                    operation::PublicKeyData::Master { .. } => None,
                    operation::PublicKeyData::Other { data, .. } => {
                        let jwk: crypto::Jwk = data.clone().into();
                        Some(json!({
                            "id": format!("{}/#{}", did, k.id),
                            "type": "JsonWebKey2020",
                            "controller": did,
                            "publicKeyJwk": {
                                "kty": jwk.kty,
                                "crv": jwk.crv,
                                "x": jwk.x,
                                "y": jwk.y,
                            }
                        }))
                    }
                }
            };
            let get_relationship = |usage: KeyUsage| -> Vec<String> {
                did_state
                    .public_keys
                    .iter()
                    .filter(|k| k.usage() == usage)
                    .map(|k| format!("{}/#{}", did, k.id))
                    .collect::<Vec<String>>()
            };
            let verification_method = did_state
                .public_keys
                .iter()
                .filter(|k| {
                    let usage = k.usage();
                    usage == KeyUsage::AuthenticationKey
                        || usage == KeyUsage::IssuingKey
                        || usage == KeyUsage::KeyAgreementKey
                        || usage == KeyUsage::CapabilityInvocationKey
                        || usage == KeyUsage::CapabilityDelegationKey
                })
                .flat_map(to_jwk_json)
                .collect();
            DidDocument {
                context,
                id: did.to_string(),
                verification_method,
                authentication: Some(get_relationship(KeyUsage::AuthenticationKey)),
                assertion_method: Some(get_relationship(KeyUsage::IssuingKey)),
                key_agreement: Some(get_relationship(KeyUsage::KeyAgreementKey)),
                capability_invocation: Some(get_relationship(KeyUsage::CapabilityInvocationKey)),
                capability_delegation: Some(get_relationship(KeyUsage::CapabilityDelegationKey)),
            }
        }
    }
}
