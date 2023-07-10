use self::model::internal::DidState;
use crate::util::Conv;
use axum::{
    extract::{Path, State},
    Json,
};
use prism_core::{prelude::CanonicalPrismDid, protocol::resolver, store::OperationStore};
use std::sync::Arc;

// TODO: make it production ready
pub async fn get_dids(
    Path(did_ref): Path<String>,
    state: State<Arc<dyn OperationStore + Send + Sync>>,
) -> Json<DidState> {
    let suffix: String = did_ref.chars().skip("did:prism:".len()).collect();
    let did = CanonicalPrismDid::from_suffix_str(&suffix).unwrap();
    let operations = state.0.get_by_did(&did).await.unwrap();
    let result = resolver::resolve(operations).unwrap();
    Json(Conv(result).into())
}

pub mod model {
    pub mod internal {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct DidState {
            pub did: String,
            pub context: Vec<String>,
            pub last_operation_hash: String,
            pub public_keys: Vec<PublicKey>,
            pub services: serde_json::Value,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct PublicKey {
            pub id: String,
            pub curve: String,
            pub data: String,
            pub key_usage: String,
        }
    }

    pub mod util {
        use crate::util::Conv;
        use prism_core::{
            crypto::{
                codec::HexStr,
                ec::{ECPublicKey, ECPublicKeyAny},
                hash::Sha256Digest,
            },
            did::{
                self,
                operation::{self, KeyUsage},
            },
            prelude::DidState,
        };

        impl From<Conv<Sha256Digest>> for String {
            fn from(value: Conv<Sha256Digest>) -> Self {
                HexStr::from(value.0.as_bytes()).to_string()
            }
        }

        impl From<Conv<ECPublicKeyAny>> for String {
            fn from(value: Conv<ECPublicKeyAny>) -> Self {
                let encoded = value.0.encode();
                HexStr::from(encoded).to_string()
            }
        }

        impl From<Conv<KeyUsage>> for String {
            fn from(value: Conv<KeyUsage>) -> Self {
                match value.0 {
                    KeyUsage::MasterKey => "Master",
                    KeyUsage::IssuingKey => "Issuing",
                    KeyUsage::KeyAgreementKey => "KeyAgreement",
                    KeyUsage::AuthenticationKey => "Authentication",
                    KeyUsage::RevocationKey => "Revocation",
                    KeyUsage::CapabilityInvocationKey => "CapabilityInvocation",
                    KeyUsage::CapabilityDelegationKey => "CapabilityDelegation",
                }
                .to_string()
            }
        }

        impl From<Conv<operation::ServiceEndpointValue>> for serde_json::Value {
            fn from(value: Conv<operation::ServiceEndpointValue>) -> Self {
                match value.0 {
                    operation::ServiceEndpointValue::URI(uri) => uri.into(),
                    operation::ServiceEndpointValue::Json(js) => js.into(),
                }
            }
        }

        impl From<Conv<operation::ServiceEndpoint>> for serde_json::Value {
            fn from(value: Conv<operation::ServiceEndpoint>) -> Self {
                match value.0 {
                    operation::ServiceEndpoint::Single(ep) => Conv(ep).into(),
                    operation::ServiceEndpoint::Multiple(eps) => eps
                        .into_iter()
                        .map(|i| serde_json::Value::from(Conv(i)))
                        .collect(),
                }
            }
        }

        impl From<Conv<operation::ServiceType>> for serde_json::Value {
            fn from(value: Conv<operation::ServiceType>) -> Self {
                match value.0 {
                    operation::ServiceType::Single(t) => t.to_string().into(),
                    operation::ServiceType::Multiple(ts) => {
                        ts.into_iter().map(|i| i.to_string()).collect()
                    }
                }
            }
        }

        impl From<Conv<operation::Service>> for serde_json::Value {
            fn from(value: Conv<operation::Service>) -> Self {
                let id = value.0.id.to_string();
                let r#type: serde_json::Value = Conv(value.0.r#type).into();
                let endpoints: serde_json::Value = Conv(value.0.service_endpoints).into();
                serde_json::json!({
                    "id": id,
                    "type": r#type,
                    "serviceEndpoint": endpoints
                })
            }
        }

        impl From<operation::PublicKey> for super::internal::PublicKey {
            fn from(value: operation::PublicKey) -> Self {
                let (key_usage, key_data) = match value.data {
                    operation::PublicKeyData::Master { data } => (KeyUsage::MasterKey, data.into()),
                    operation::PublicKeyData::Other { data, usage } => (usage, data),
                };

                super::internal::PublicKey {
                    id: value.id.to_string(),
                    curve: key_data.curve_name().to_string(),
                    data: Conv(key_data).into(),
                    key_usage: Conv(key_usage).into(),
                }
            }
        }

        impl From<Conv<did::DidState>> for super::internal::DidState {
            fn from(value: Conv<DidState>) -> Self {
                super::internal::DidState {
                    did: value.0.did.to_string(),
                    context: value.0.context,
                    last_operation_hash: Conv(value.0.last_operation_hash).into(),
                    public_keys: value
                        .0
                        .public_keys
                        .into_iter()
                        .map(|pk| pk.into())
                        .collect(),
                    services: value
                        .0
                        .services
                        .into_iter()
                        .map(|s| serde_json::Value::from(Conv(s)))
                        .collect(),
                }
            }
        }
    }
}
