use prism_core::store::{CursorStoreError, OperationStoreError};
use sea_orm::DbErr;

pub(crate) struct Conv<T: 'static>(pub T);

#[cfg(feature = "sqlite")]
mod sqlite {
    use super::Conv;
    use crate::entity;
    use chrono::{DateTime, Utc};
    use prism_core::{
        dlt::{BlockMetadata, OperationMetadata},
        prelude::*,
        proto::SignedAtalaOperation,
        store::{self, DltCursor},
        util::StdError,
    };
    use sea_orm::ActiveValue;

    impl From<Conv<DateTime<Utc>>> for String {
        fn from(value: Conv<DateTime<Utc>>) -> Self {
            value.0.to_rfc3339()
        }
    }

    impl TryFrom<Conv<String>> for DateTime<Utc> {
        type Error = Box<dyn std::error::Error>;

        fn try_from(value: Conv<String>) -> Result<Self, Self::Error> {
            Ok(DateTime::parse_from_rfc3339(&value.0)?.with_timezone(&Utc))
        }
    }

    impl TryFrom<entity::raw_operation::Model> for (OperationMetadata, SignedAtalaOperation) {
        type Error = StdError;

        fn try_from(value: entity::raw_operation::Model) -> Result<Self, Self::Error> {
            let metadata = OperationMetadata {
                block_metadata: BlockMetadata {
                    slot_number: value.slot.try_into()?,
                    block_number: value.block_number.try_into()?,
                    cbt: Conv(value.cbt).try_into()?,
                    absn: value.absn.try_into()?,
                },
                osn: value.osn.try_into()?,
            };
            let bytes: &[u8] = &value.signed_operation_data;
            let operation = SignedAtalaOperation::decode(bytes)?;
            Ok((metadata, operation))
        }
    }

    impl TryFrom<(OperationMetadata, SignedAtalaOperation)> for entity::raw_operation::ActiveModel {
        type Error = StdError;

        fn try_from(value: (OperationMetadata, SignedAtalaOperation)) -> Result<Self, Self::Error> {
            let metadata = value.0;
            let signed_operation = value.1;
            let did = store::get_did_from_signed_operation(&signed_operation)?;
            let operation = entity::raw_operation::ActiveModel {
                slot: ActiveValue::Set(metadata.block_metadata.slot_number.try_into()?),
                block_number: ActiveValue::Set(metadata.block_metadata.block_number.try_into()?),
                did: ActiveValue::Set(did.suffix.as_bytes().to_vec()),
                signed_operation_data: ActiveValue::Set(signed_operation.encode_to_bytes()?.into()),
                cbt: ActiveValue::Set(Conv(metadata.block_metadata.cbt).into()),
                absn: ActiveValue::Set(metadata.block_metadata.absn.try_into()?),
                osn: ActiveValue::Set(metadata.osn.try_into()?),
            };
            Ok(operation)
        }
    }

    impl TryFrom<entity::dlt_cursor::Model> for DltCursor {
        type Error = StdError;

        fn try_from(value: entity::dlt_cursor::Model) -> Result<Self, Self::Error> {
            Ok(Self {
                slot: value.slot.try_into()?,
                block_hash: value.block_hash,
            })
        }
    }
}

impl From<Conv<DbErr>> for CursorStoreError {
    fn from(value: Conv<DbErr>) -> Self {
        CursorStoreError::StorageBackendError(value.0.into())
    }
}

impl From<Conv<DbErr>> for OperationStoreError {
    fn from(value: Conv<DbErr>) -> Self {
        OperationStoreError::StorageBackendError(value.0.into())
    }
}
