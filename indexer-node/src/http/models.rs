use identus_apollo::jwk::EncodeJwk;
use identus_did_core::{
    DidDocument, Service, ServiceEndpoint, ServiceType, StringOrMap, VerificationMethod, VerificationMethodOrRef,
};
use identus_did_prism::did::operation::KeyUsage;
use identus_did_prism::did::{DidState, operation};

pub fn new_did_document(did: &str, did_state: &DidState) -> DidDocument {
    let mut context = vec!["https://www.w3.org/ns/did/v1".to_string()];
    context.extend(did_state.context.clone());

    let get_relationship = |usage: KeyUsage| -> Vec<VerificationMethodOrRef> {
        did_state
            .public_keys
            .iter()
            .filter(|k| k.usage() == usage)
            .map(|k| VerificationMethodOrRef::Ref(format!("{}#{}", did, k.id)))
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

fn transform_key_jwk(did: &str, key: &operation::PublicKey) -> Option<VerificationMethod> {
    match &key.data {
        operation::PublicKeyData::Master { .. } => None,
        operation::PublicKeyData::Other { data, .. } => {
            let jwk = data.encode_jwk();
            Some(VerificationMethod {
                id: format!("{}#{}", did, key.id),
                r#type: "JsonWebKey2020".to_string(),
                controller: did.to_string(),
                public_key_jwk: Some(jwk),
            })
        }
    }
}

fn transform_service(service: &operation::Service) -> Service {
    let r#type = match &service.r#type {
        operation::ServiceType::Value(name) => ServiceType::Str(name.to_string()),
        operation::ServiceType::List(names) => ServiceType::List(names.iter().map(|i| i.to_string()).collect()),
    };
    let transform_endpoint_value = |uri: &operation::ServiceEndpointValue| -> StringOrMap {
        match &uri {
            operation::ServiceEndpointValue::Uri(uri) => StringOrMap::Str(uri.to_string()),
            operation::ServiceEndpointValue::Json(obj) => StringOrMap::Map(obj.clone()),
        }
    };
    let service_endpoint = match &service.service_endpoint {
        operation::ServiceEndpoint::Value(endpoint) => ServiceEndpoint::StrOrMap(transform_endpoint_value(endpoint)),
        operation::ServiceEndpoint::List(endpoints) => {
            ServiceEndpoint::List(endpoints.iter().map(transform_endpoint_value).collect())
        }
    };
    Service {
        id: service.id.to_string(),
        r#type,
        service_endpoint,
    }
}
