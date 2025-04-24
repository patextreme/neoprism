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
        GetExplorerDidList { page: Option<u32> },
    }
}

pub mod api {
    use prism_core::crypto::EncodeJwk;
    use prism_core::did::operation::KeyUsage;
    use prism_core::did::{DidState, operation};
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
        service: Option<Vec<Service>>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(crate = "rocket::serde", rename_all = "camelCase")]
    pub struct Service {
        id: String,
        r#type: serde_json::Value,
        service_endpoint: serde_json::Value,
    }

    impl DidDocument {
        pub fn new(did: &str, did_state: DidState) -> Self {
            let mut context = vec!["https://www.w3.org/ns/did/v1".to_string()];
            context.extend(did_state.context);

            let get_relationship = |usage: KeyUsage| -> Vec<String> {
                did_state
                    .public_keys
                    .iter()
                    .filter(|k| k.usage() == usage)
                    .map(|k| format!("{}#{}", did, k.id))
                    .collect()
            };
            let verification_method = did_state
                .public_keys
                .iter()
                .filter(|k| {
                    const W3C_KEY_TYPES: [KeyUsage; 5] = [
                        KeyUsage::AuthenticationKey,
                        KeyUsage::IssuingKey,
                        KeyUsage::KeyAgreementKey,
                        KeyUsage::CapabilityInvocationKey,
                        KeyUsage::CapabilityDelegationKey,
                    ];
                    W3C_KEY_TYPES.iter().any(|usage| usage == &k.usage())
                })
                .flat_map(|k| transform_key_jwk(did, k))
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
                service: Some(did_state.services.iter().map(transform_service).collect()),
            }
        }
    }

    fn transform_key_jwk(did: &str, key: &operation::PublicKey) -> Option<serde_json::Value> {
        match &key.data {
            operation::PublicKeyData::Master { .. } => None,
            operation::PublicKeyData::Other { data, .. } => {
                let jwk = data.encode_jwk();
                Some(json!({
                    "id": format!("{}#{}", did, key.id),
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
    }

    fn transform_service(service: &operation::Service) -> Service {
        let r#type = match &service.r#type {
            operation::ServiceType::Value(name) => json!(name.to_string()),
            operation::ServiceType::List(names) => {
                json!(names.iter().map(|i| i.to_string()).collect::<Vec<_>>())
            }
        };
        let endpoint_to_json = |uri: &operation::ServiceEndpointValue| -> serde_json::Value {
            match &uri {
                operation::ServiceEndpointValue::Uri(uri) => json!(uri),
                operation::ServiceEndpointValue::Json(obj) => json!(obj),
            }
        };
        let service_endpoint = match &service.service_endpoint {
            operation::ServiceEndpoint::Value(endpoint) => endpoint_to_json(endpoint),
            operation::ServiceEndpoint::List(endpoints) => {
                json!(endpoints.iter().map(endpoint_to_json).collect::<Vec<_>>())
            }
        };
        Service {
            id: service.id.to_string(),
            r#type,
            service_endpoint,
        }
    }
}
