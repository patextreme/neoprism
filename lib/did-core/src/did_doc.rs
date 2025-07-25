use identus_apollo::jwk::Jwk;
use serde::{Deserialize, Serialize};

use crate::Did;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct DidDocument {
    #[serde(rename(serialize = "@context", deserialize = "@context"))]
    pub context: Vec<String>,
    pub id: Did,
    pub verification_method: Vec<VerificationMethod>,
    pub authentication: Option<Vec<VerificationMethodOrRef>>,
    pub assertion_method: Option<Vec<VerificationMethodOrRef>>,
    pub key_agreement: Option<Vec<VerificationMethodOrRef>>,
    pub capability_invocation: Option<Vec<VerificationMethodOrRef>>,
    pub capability_delegation: Option<Vec<VerificationMethodOrRef>>,
    pub service: Option<Vec<Service>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct VerificationMethod {
    pub id: String,
    pub r#type: String,
    pub controller: String,
    pub public_key_jwk: Option<Jwk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum VerificationMethodOrRef {
    Embedded(VerificationMethod),
    Ref(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub id: String,
    pub r#type: ServiceType,
    pub service_endpoint: ServiceEndpoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum ServiceType {
    Str(String),
    List(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum ServiceEndpoint {
    StrOrMap(StringOrMap),
    List(Vec<StringOrMap>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum StringOrMap {
    Str(String),
    Map(serde_json::Map<String, serde_json::Value>),
}
