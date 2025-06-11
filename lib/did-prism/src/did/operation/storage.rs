use identus_apollo::hash::Sha256Digest;
use identus_apollo::hex::HexStr;

use crate::did::CanonicalPrismDid;
use crate::did::error::{CreateStorageOperationError, DeactivateStorageOperationError, UpdateStorageOperationError};
use crate::proto::proto_create_storage_entry::Data as ProtoCreateStorageData;
use crate::proto::proto_update_storage_entry::Data as ProtoUpdateStorageData;
use crate::proto::{ProtoCreateStorageEntry, ProtoDeactivateStorageEntry, ProtoUpdateStorageEntry};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusListData {
    pub state: i64,
    pub name: String,
    pub detail: String,
}

#[derive(Clone, PartialEq, Eq, derive_more::Debug)]
pub enum StorageData {
    #[debug("bytes({})", identus_apollo::hex::HexStr::from(_0))]
    Bytes(Vec<u8>),
    #[debug("ipfs({_0})")]
    Ipfs(String),
    #[debug("status_list({_0:?})")]
    StatusList(StatusListData),
}

impl From<&ProtoCreateStorageData> for StorageData {
    fn from(value: &ProtoCreateStorageData) -> Self {
        match value {
            ProtoCreateStorageData::Bytes(bytes) => StorageData::Bytes(bytes.clone()),
            ProtoCreateStorageData::Ipfs(cid) => StorageData::Ipfs(cid.clone()),
            ProtoCreateStorageData::StatusListEntry(sle) => StorageData::StatusList(StatusListData {
                state: sle.state,
                name: sle.name.clone(),
                detail: sle.details.clone(),
            }),
        }
    }
}

impl From<&ProtoUpdateStorageData> for StorageData {
    fn from(value: &ProtoUpdateStorageData) -> Self {
        match value {
            ProtoUpdateStorageData::Bytes(bytes) => StorageData::Bytes(bytes.clone()),
            ProtoUpdateStorageData::Ipfs(cid) => StorageData::Ipfs(cid.clone()),
            ProtoUpdateStorageData::StatusListEntry(sle) => StorageData::StatusList(StatusListData {
                state: sle.state,
                name: sle.name.clone(),
                detail: sle.details.clone(),
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreateStorageOperation {
    pub id: CanonicalPrismDid,
    pub nonce: Vec<u8>,
    pub data: StorageData,
}

impl CreateStorageOperation {
    pub fn parse(operation: &ProtoCreateStorageEntry) -> Result<Self, CreateStorageOperationError> {
        let suffix = HexStr::from(&operation.did_prism_hash);
        let id = CanonicalPrismDid::from_suffix(suffix)?;
        let data = operation
            .data
            .as_ref()
            .ok_or(CreateStorageOperationError::EmptyStorageData)?
            .into();

        Ok(Self {
            id,
            nonce: operation.nonce.clone(),
            data,
        })
    }
}

#[derive(Debug, Clone)]
pub struct UpdateStorageOperation {
    pub prev_operation_hash: Sha256Digest,
    pub data: StorageData,
}

impl UpdateStorageOperation {
    pub fn parse(operation: &ProtoUpdateStorageEntry) -> Result<Self, UpdateStorageOperationError> {
        let prev_operation_hash = Sha256Digest::from_bytes(&operation.previous_operation_hash)
            .map_err(|e| UpdateStorageOperationError::InvalidPreviousOperationHash { source: e })?;
        let data = operation
            .data
            .as_ref()
            .ok_or(UpdateStorageOperationError::EmptyStorageData)?
            .into();

        Ok(Self {
            prev_operation_hash,
            data,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DeactivateStorageOperation {
    pub prev_operation_hash: Sha256Digest,
}

impl DeactivateStorageOperation {
    pub fn parse(operation: &ProtoDeactivateStorageEntry) -> Result<Self, DeactivateStorageOperationError> {
        let prev_operation_hash = Sha256Digest::from_bytes(&operation.previous_operation_hash)
            .map_err(|e| DeactivateStorageOperationError::InvalidPreviousOperationHash { source: e })?;
        Ok(Self { prev_operation_hash })
    }
}
