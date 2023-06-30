use super::{CanonicalPrismDid, DidParsingError};
use crate::{
    crypto::{
        ec::{ECPublicKeyAny, Secp256k1PublicKey},
        hash::Sha256Digest,
    },
    proto::{
        self, update_did_action::Action, CreateDidOperation, DeactivateDidOperation,
        UpdateDidAction, UpdateDidOperation,
    },
    protocol::ProtocolParameter,
    util::{is_slice_unique, is_uri_fragment},
};
use bytes::Bytes;

#[derive(Debug, Clone, thiserror::Error)]
pub enum CreateOperationParsingError {
    #[error("Missing did_data in create operation")]
    MissingDidData,
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(#[from] PublicKeyParsingError),
    #[error("Invalid service: {0}")]
    InvalidService(#[from] ServiceParsingError),
    #[error("Too many public keys")]
    TooManyPublicKeys,
    #[error("No master key found")]
    NoMasterKey,
    #[error("Too many services")]
    TooManyServices,
    #[error("Duplicate context")]
    DuplicateContext,
}

#[derive(Debug, Clone)]
pub struct CreateOperation {
    pub public_keys: Vec<PublicKey>,
    pub services: Vec<Service>,
    pub context: Vec<String>,
}

impl CreateOperation {
    pub fn parse(
        param: &ProtocolParameter,
        operation: &CreateDidOperation,
    ) -> Result<Self, CreateOperationParsingError> {
        let Some(did_data) = &operation.did_data else {
            Err(CreateOperationParsingError::MissingDidData)?
        };

        let public_keys = did_data
            .public_keys
            .iter()
            .map(|pk| PublicKey::parse(pk, param))
            .collect::<Result<Vec<_>, _>>()?;
        let services = did_data
            .services
            .iter()
            .map(|s| Service::parse(s, param))
            .collect::<Result<Vec<_>, _>>()?;
        let context = did_data.context.clone();

        Self::validate_public_key_list(param, &public_keys)?;
        Self::validate_service_list(param, &services)?;
        Self::validate_context_list(&context)?;

        Ok(Self {
            public_keys,
            services,
            context,
        })
    }

    fn validate_public_key_list(
        param: &ProtocolParameter,
        public_keys: &[PublicKey],
    ) -> Result<(), CreateOperationParsingError> {
        if public_keys.len() > param.max_public_keys {
            Err(CreateOperationParsingError::TooManyPublicKeys)?
        }

        if !public_keys.iter().any(|i| i.usage() == KeyUsage::MasterKey) {
            Err(CreateOperationParsingError::NoMasterKey)?
        }

        Ok(())
    }

    fn validate_service_list(
        param: &ProtocolParameter,
        services: &[Service],
    ) -> Result<(), CreateOperationParsingError> {
        if services.len() > param.max_services {
            return Err(CreateOperationParsingError::TooManyServices);
        }

        Ok(())
    }

    fn validate_context_list(contexts: &[String]) -> Result<(), CreateOperationParsingError> {
        if is_slice_unique(contexts) {
            Ok(())
        } else {
            Err(CreateOperationParsingError::DuplicateContext)?
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum UpdateOperationParsingError {
    #[error("Invalid did id: {0}")]
    InvalidDidId(#[from] DidParsingError),
    #[error("Invalid previous operation hash: {0}")]
    InvalidPreviousOperationHash(String),
    #[error("Update action is malformed: {0}")]
    MalformedUpdateAction(String),
    #[error("Unable to parse public_key in update action: {0}")]
    PublicKeyParsingError(#[from] PublicKeyParsingError),
    #[error("Unable to parse service in update action: {0}")]
    ServiceParsingError(#[from] ServiceParsingError),
    #[error("Unable to parse service_type in update action: {0}")]
    ServiceTypeParsingError(String),
    #[error("Unable to parse service_endpoint in update action: {0}")]
    ServiceEndpointParsingError(String),
    #[error("Empty update action")]
    EmptyUpdateAction,
}

#[derive(Debug, Clone)]
pub struct UpdateOperation {
    pub id: CanonicalPrismDid,
    pub prev_operation_hash: Sha256Digest,
    pub actions: Vec<UpdateOperationAction>,
}

impl UpdateOperation {
    pub fn parse(
        param: &ProtocolParameter,
        operation: &UpdateDidOperation,
    ) -> Result<Self, UpdateOperationParsingError> {
        if operation.actions.is_empty() {
            Err(UpdateOperationParsingError::EmptyUpdateAction)?
        }

        let id = CanonicalPrismDid::from_suffix_str(&operation.id)?;
        let prev_operation_hash = Bytes::copy_from_slice(&operation.previous_operation_hash);
        let prev_operation_hash = Sha256Digest::from_bytes(prev_operation_hash)
            .map_err(UpdateOperationParsingError::InvalidPreviousOperationHash)?;

        let actions = operation
            .actions
            .iter()
            .map(|action| UpdateOperationAction::parse(action, param))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();

        Ok(Self {
            id,
            prev_operation_hash,
            actions,
        })
    }
}

#[derive(Debug, Clone)]
pub enum UpdateOperationAction {
    AddKey(PublicKey),
    RemoveKey(PublicKeyId),
    AddService(Service),
    RemoveService(ServiceId),
    UpdateService {
        id: ServiceId,
        r#type: Option<ServiceType>,
        service_endpoints: Option<ServiceEndpoint>,
    },
    PatchContext(Vec<String>),
}

impl UpdateOperationAction {
    pub fn parse(
        action: &UpdateDidAction,
        param: &ProtocolParameter,
    ) -> Result<Option<Self>, UpdateOperationParsingError> {
        let Some(action) = &action.action else {
            return Ok(None)
        };

        let action = match action {
            Action::AddKey(add_key) => match &add_key.key {
                Some(pk) => {
                    let parsed_key = PublicKey::parse(pk, param)?;
                    Self::AddKey(parsed_key)
                }
                None => Err(UpdateOperationParsingError::MalformedUpdateAction(
                    "AddKey action must have key property".to_owned(),
                ))?,
            },
            Action::RemoveKey(remove_key) => {
                let key_id =
                    PublicKeyId::parse(&remove_key.key_id, param.max_id_size).map_err(|e| {
                        UpdateOperationParsingError::MalformedUpdateAction(format!(
                            "Public key id cannot be parsed ({})",
                            e
                        ))
                    })?;
                Self::RemoveKey(key_id)
            }
            Action::AddService(add_service) => match &add_service.service {
                Some(service) => {
                    let parsed_service = Service::parse(service, param)?;
                    Self::AddService(parsed_service)
                }
                None => Err(UpdateOperationParsingError::MalformedUpdateAction(
                    "AddService action must have service property".to_owned(),
                ))?,
            },
            Action::RemoveService(remove_service) => {
                let service_id = ServiceId::parse(&remove_service.service_id, param.max_id_size)
                    .map_err(|_| {
                        UpdateOperationParsingError::MalformedUpdateAction(
                            "Service id cannot be parsed".to_string(),
                        )
                    })?;
                Self::RemoveService(service_id)
            }
            Action::UpdateService(update_service) => {
                let service_id = ServiceId::parse(&update_service.service_id, param.max_id_size)
                    .map_err(|_| {
                        UpdateOperationParsingError::MalformedUpdateAction(
                            "Service id cannot be parsed".to_string(),
                        )
                    })?;
                let service_type =
                    match Some(update_service.r#type.clone()).filter(|i| !i.is_empty()) {
                        Some(s) => Some(
                            ServiceType::parse(&s, param)
                                .map_err(UpdateOperationParsingError::ServiceTypeParsingError)?,
                        ),
                        None => None,
                    };
                let service_endpoints = match Some(update_service.service_endpoints.clone())
                    .filter(|i| !i.is_empty())
                {
                    Some(s) => Some(
                        ServiceEndpoint::parse(&s, param)
                            .map_err(UpdateOperationParsingError::ServiceEndpointParsingError)?,
                    ),
                    None => None,
                };
                Self::UpdateService {
                    id: service_id,
                    r#type: service_type,
                    service_endpoints,
                }
            }
            Action::PatchContext(patch_ctx) => Self::PatchContext(patch_ctx.context.clone()),
        };

        Ok(Some(action))
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum DeactivateOperationParsingError {
    #[error("Invalid did id: {0}")]
    InvalidDidId(#[from] DidParsingError),
    #[error("Invalid previous operation hash: {0}")]
    InvalidPreviousOperationHash(String),
}

#[derive(Debug, Clone)]
pub struct DeactivateOperation {
    pub id: CanonicalPrismDid,
    pub prev_operation_hash: Sha256Digest,
}

impl DeactivateOperation {
    pub fn parse(
        operation: &DeactivateDidOperation,
    ) -> Result<Self, DeactivateOperationParsingError> {
        let id = CanonicalPrismDid::from_suffix_str(&operation.id)?;
        let prev_operation_hash = Bytes::copy_from_slice(&operation.previous_operation_hash);
        let prev_operation_hash = Sha256Digest::from_bytes(prev_operation_hash)
            .map_err(DeactivateOperationParsingError::InvalidPreviousOperationHash)?;

        Ok(Self {
            id,
            prev_operation_hash,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PublicKeyId(String);

impl PublicKeyId {
    pub fn parse(id: &str, max_length: usize) -> Result<Self, String> {
        let is_fragment = is_uri_fragment(id);
        let is_non_empty = !id.is_empty();
        let is_within_max_size = id.len() <= max_length;
        if is_fragment && is_non_empty && is_within_max_size {
            Ok(Self(id.to_owned()))
        } else {
            Err(format!(
                "Public key id must be non-empty URI fragment with max length of {}. Got {}",
                max_length, id
            ))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PublicKeyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServiceId(String);

impl ServiceId {
    pub fn parse(id: &str, max_length: usize) -> Result<Self, String> {
        let is_fragment = is_uri_fragment(id);
        let is_non_empty = !id.is_empty();
        let is_within_max_size = id.len() <= max_length;
        if is_fragment && is_non_empty && is_within_max_size {
            Ok(Self(id.to_owned()))
        } else {
            Err(format!(
                "Service id must be non-empty URI fragment with max length of {}. Got {}",
                max_length, id
            ))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ServiceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum PublicKeyParsingError {
    #[error("Invalid key id: {0}")]
    InvalidKeyId(String),
    #[error("Missing key_data on key id {0:?}")]
    MissingKeyData(PublicKeyId),
    #[error("Unknown key usage on key id {0:?}")]
    UnknownKeyUsage(PublicKeyId),
    #[error("Unable to parse key_data on key id {id:?}: {msg}")]
    KeyDataParseError { id: PublicKeyId, msg: String },
    #[error("Master key must have type of secp256k1. (id {0:?}) ")]
    InvalidMasterKeyType(PublicKeyId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey {
    pub id: PublicKeyId,
    pub data: PublicKeyData,
}

impl PublicKey {
    pub fn parse(
        public_key: &proto::PublicKey,
        param: &ProtocolParameter,
    ) -> Result<Self, PublicKeyParsingError> {
        let id = PublicKeyId::parse(&public_key.id, param.max_id_size)
            .map_err(PublicKeyParsingError::InvalidKeyId)?;
        let usage = KeyUsage::parse(&public_key.usage())
            .ok_or(PublicKeyParsingError::UnknownKeyUsage(id.clone()))?;
        let Some(key_data) = &public_key.key_data else {
            Err(PublicKeyParsingError::MissingKeyData(id))?
        };
        let pk = ECPublicKeyAny::from_key_data(key_data).map_err(|e| {
            PublicKeyParsingError::KeyDataParseError {
                id: id.clone(),
                msg: e,
            }
        })?;
        let data = match (usage, pk) {
            (KeyUsage::MasterKey, ECPublicKeyAny::Secp256k1(pk)) => {
                PublicKeyData::Master { data: pk }
            }
            (KeyUsage::MasterKey, _) => {
                Err(PublicKeyParsingError::InvalidMasterKeyType(id.clone()))?
            }
            (usage, pk) => PublicKeyData::Other { data: pk, usage },
        };

        Ok(Self { id, data })
    }

    pub fn usage(&self) -> KeyUsage {
        match &self.data {
            PublicKeyData::Master { .. } => KeyUsage::MasterKey,
            PublicKeyData::Other { usage, .. } => *usage,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicKeyData {
    Master {
        data: Secp256k1PublicKey,
    },
    Other {
        data: ECPublicKeyAny,
        usage: KeyUsage,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyUsage {
    MasterKey,
    IssuingKey,
    KeyAgreementKey,
    AuthenticationKey,
    RevocationKey,
    CapabilityInvocationKey,
    CapabilityDelegationKey,
}

impl KeyUsage {
    pub fn parse(usage: &proto::KeyUsage) -> Option<Self> {
        match usage {
            proto::KeyUsage::MasterKey => Some(Self::MasterKey),
            proto::KeyUsage::IssuingKey => Some(Self::IssuingKey),
            proto::KeyUsage::KeyAgreementKey => Some(Self::KeyAgreementKey),
            proto::KeyUsage::AuthenticationKey => Some(Self::AuthenticationKey),
            proto::KeyUsage::RevocationKey => Some(Self::RevocationKey),
            proto::KeyUsage::CapabilityInvocationKey => Some(Self::CapabilityInvocationKey),
            proto::KeyUsage::CapabilityDelegationKey => Some(Self::CapabilityDelegationKey),
            proto::KeyUsage::UnknownKey => None,
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ServiceParsingError {
    #[error("Invalid service id: {0}")]
    InvalidServiceId(String),
    #[error("Invalid service_type: {0}")]
    InvalidServiceType(String),
    #[error("Invalid service_endpoint: {0}")]
    InvalidServiceEndpoint(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Service {
    pub id: ServiceId,
    pub r#type: ServiceType,
    pub service_endpoints: ServiceEndpoint,
}

impl Service {
    pub fn parse(
        service: &proto::Service,
        param: &ProtocolParameter,
    ) -> Result<Self, ServiceParsingError> {
        let id = ServiceId::parse(&service.id, param.max_id_size)
            .map_err(ServiceParsingError::InvalidServiceId)?;
        let r#type = ServiceType::parse(&service.r#type, param)
            .map_err(ServiceParsingError::InvalidServiceType)?;
        let service_endpoints = ServiceEndpoint::parse(&service.service_endpoint, param)
            .map_err(ServiceParsingError::InvalidServiceEndpoint)?;

        Ok(Self {
            id,
            r#type,
            service_endpoints,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceType {
    Single(String),
    Multiple(Vec<String>),
}

impl ServiceType {
    // TODO: align with the spec
    pub fn parse(service_type: &str, param: &ProtocolParameter) -> Result<Self, String> {
        if service_type.len() > param.max_type_size {
            Err(format!(
                "Service type must not exceed {} bytes. Got {} bytes",
                param.max_type_size,
                service_type.len()
            ))?
        }

        // try parse as json list of strings
        let parsed: Result<Vec<String>, _> = serde_json::from_str(service_type);
        if let Ok(list) = parsed {
            if list.is_empty() {
                Err("Service type must not be empty".to_owned())?
            }

            for i in &list {
                Self::validate_type_value(i)?;
            }

            return Ok(Self::Multiple(list));
        }

        // try parse as single string
        Self::validate_type_value(service_type)?;
        Ok(Self::Single(service_type.to_owned()))
    }

    fn validate_type_value(value: &str) -> Result<(), String> {
        if value.is_empty() {
            Err("Service type must not be empty".to_owned())?
        }

        if value.starts_with(char::is_whitespace) || value.ends_with(char::is_whitespace) {
            Err(format!(
                "Service type must not start or end with whitespace. Got '{}'",
                value
            ))?
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceEndpoint {
    Single(ServiceEndpointValue),
    Multiple(Vec<ServiceEndpointValue>),
}

impl ServiceEndpoint {
    // TODO: align with the spec
    pub fn parse(service_endpoint: &str, param: &ProtocolParameter) -> Result<Self, String> {
        if service_endpoint.len() > param.max_service_endpoint_size {
            Err(format!(
                "Service endpoint must not exceed {} bytes. Got {} bytes",
                param.max_service_endpoint_size,
                service_endpoint.len()
            ))?
        }

        // try parsing as json object
        let parsed_map: Result<serde_json::Map<String, serde_json::Value>, _> =
            serde_json::from_str(service_endpoint);
        if let Ok(json) = parsed_map {
            return Ok(Self::Single(ServiceEndpointValue::Json(json)));
        }

        // try parsing as json array
        let parsed_array: Result<Vec<serde_json::Value>, _> =
            serde_json::from_str(service_endpoint);
        if let Ok(list) = parsed_array {
            if list.is_empty() {
                Err("Service endpoint must not be empty array".to_owned())?
            }

            let mut endpoints = Vec::with_capacity(list.len());
            for i in list {
                endpoints.push(ServiceEndpointValue::parse(i)?);
            }

            return Ok(Self::Multiple(endpoints));
        }

        // try parse as single string
        Ok(Self::Single(ServiceEndpointValue::URI(
            service_endpoint.to_owned(),
        )))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceEndpointValue {
    URI(String), // TODO: validate value is a valid normalized URI
    Json(serde_json::Map<String, serde_json::Value>),
}

impl ServiceEndpointValue {
    pub fn parse(value: serde_json::Value) -> Result<Self, String> {
        match value {
            serde_json::Value::String(s) => Ok(Self::URI(s)),
            serde_json::Value::Object(map) => Ok(Self::Json(map)),
            _ => Err(format!(
                "Service endpoint must be either a URI or a JSON object. Got '{}'",
                value
            )),
        }
    }
}
