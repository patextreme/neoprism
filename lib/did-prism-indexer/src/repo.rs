use identus_apollo::hash::Sha256Digest;
use identus_did_prism::did::CanonicalPrismDid;
use identus_did_prism::dlt::{DltCursor, OperationMetadata};
use identus_did_prism::proto::SignedPrismOperation;
use identus_did_prism::utils::paging::Paginated;
use uuid::Uuid;

#[derive(Clone, Debug, Copy, derive_more::From, derive_more::Into, derive_more::AsRef)]
pub struct RawOperationId(Uuid);

pub enum IndexedOperation {
    Ssi {
        raw_operation_id: RawOperationId,
        did: CanonicalPrismDid,
    },
    Vdr {
        raw_operation_id: RawOperationId,
        operation_hash: Vec<u8>,
        init_operation_hash: Vec<u8>,
        prev_operation_hash: Option<Vec<u8>>,
        did: CanonicalPrismDid,
    },
    Ignored {
        raw_operation_id: RawOperationId,
    },
}

impl IndexedOperation {
    pub fn raw_operation_id(&self) -> &RawOperationId {
        match self {
            IndexedOperation::Ssi { raw_operation_id, .. } => raw_operation_id,
            IndexedOperation::Vdr { raw_operation_id, .. } => raw_operation_id,
            IndexedOperation::Ignored { raw_operation_id } => raw_operation_id,
        }
    }
}

#[async_trait::async_trait]
pub trait OperationRepo {
    type Error: std::error::Error;

    async fn get_all_dids(&self, page: u32, page_size: u32) -> Result<Paginated<CanonicalPrismDid>, Self::Error>;

    async fn get_unindexed_raw_operations(
        &self,
    ) -> Result<Vec<(RawOperationId, OperationMetadata, SignedPrismOperation)>, Self::Error>;

    async fn get_raw_operations_by_did(
        &self,
        did: &CanonicalPrismDid,
    ) -> Result<Vec<(RawOperationId, OperationMetadata, SignedPrismOperation)>, Self::Error>;

    async fn insert_raw_operations(
        &self,
        operations: Vec<(OperationMetadata, SignedPrismOperation)>,
    ) -> Result<(), Self::Error>;

    async fn insert_indexed_operations(&self, operations: Vec<IndexedOperation>) -> Result<(), Self::Error>;

    async fn get_vdr_raw_operation_by_operation_hash(
        &self,
        operation_hash: &Sha256Digest,
    ) -> Result<Option<(RawOperationId, OperationMetadata, SignedPrismOperation)>, Self::Error>;
}

#[async_trait::async_trait]
pub trait DltCursorRepo {
    type Error: std::error::Error;

    async fn set_cursor(&self, cursor: DltCursor) -> Result<(), Self::Error>;
    async fn get_cursor(&self) -> Result<Option<DltCursor>, Self::Error>;
}
