use prism_core::{
    dlt::{BlockMetadata, OperationMetadata},
    prelude::Message,
    proto::SignedAtalaOperation,
    util::StdError,
};
use sea_query::Iden;
use time::OffsetDateTime;

#[derive(Iden)]
pub enum RawOperation {
    Table,
    Did,
    SignedOperationData,
    Slot,
    BlockNumber,
    Cbt,
    Absn,
    Osn,
}

#[derive(Debug, sqlx::FromRow)]
pub struct RawOperationRow {
    pub did: String,
    pub signed_operation_data: Vec<u8>,
    pub slot: i64,
    pub block_number: i64,
    pub cbt: OffsetDateTime,
    pub absn: i64,
    pub osn: i64,
}

impl TryFrom<RawOperationRow> for (OperationMetadata, SignedAtalaOperation) {
    type Error = StdError;

    fn try_from(value: RawOperationRow) -> Result<Self, Self::Error> {
        let metadata = OperationMetadata {
            block_metadata: BlockMetadata {
                slot_number: value.slot.try_into()?,
                block_number: value.block_number.try_into()?,
                cbt: value.cbt,
                absn: value.absn.try_into()?,
            },
            osn: value.osn.try_into()?,
        };
        let bytes: &[u8] = &value.signed_operation_data;
        let operation = SignedAtalaOperation::decode(bytes)?;
        Ok((metadata, operation))
    }
}

#[derive(Iden)]
pub enum DltCursor {
    Table,
    Slot,
    BlockHash,
}

#[derive(Debug, sqlx::FromRow)]
pub struct DltCursorRow {
    slot: i64,
    block_hash: Vec<u8>,
}

impl TryFrom<DltCursorRow> for prism_core::store::DltCursor {
    type Error = StdError;

    fn try_from(value: DltCursorRow) -> Result<Self, Self::Error> {
        Ok(prism_core::store::DltCursor {
            slot: value.slot.try_into()?,
            block_hash: value.block_hash.try_into()?,
        })
    }
}
