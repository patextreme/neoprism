#![feature(error_generic_member_access)]

use std::backtrace::Backtrace;

use prism_core::did::operation::get_did_from_signed_operation;
use prism_core::did::Error as DidError;
use prism_core::dlt::{BlockMetadata, DltCursor, OperationMetadata};
use prism_core::prelude::*;
use prism_core::proto::SignedAtalaOperation;
use prism_core::store::{DltCursorStore, OperationStore};
use prism_core::utils::codec::HexStr;
use sea_orm::{
    ColumnTrait, ConnectOptions, Database, DatabaseConnection, DatabaseTransaction, EntityTrait, FromQueryResult,
    IntoActiveValue, ModelTrait, QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};
use sea_query::{Alias, Expr, OnConflict};

mod entity;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {source}")]
    Db {
        #[from]
        source: sea_orm::DbErr,
        backtrace: Backtrace,
    },
    #[error("Unable to decode to protobuf message. {source}")]
    ProtobufDecode {
        #[from]
        source: prost::DecodeError,
        backtrace: Backtrace,
    },
    #[error("{source}")]
    Did {
        #[from]
        source: DidError,
        backtrace: Backtrace,
    },
}

#[derive(Debug, Clone)]
pub struct PostgresDb {
    db: DatabaseConnection,
}

impl PostgresDb {
    pub async fn connect(url: &str, log_statement: bool) -> Result<Self, Error> {
        let mut options = ConnectOptions::new(url);
        options.sqlx_logging(log_statement);
        Ok(Self {
            db: Database::connect(options).await?,
        })
    }

    pub async fn begin(&self) -> Result<PostgresTransaction, Error> {
        let tx = self.db.begin().await?;
        Ok(PostgresTransaction { tx })
    }
}

pub struct PostgresTransaction {
    tx: DatabaseTransaction,
}

impl PostgresTransaction {
    pub async fn commit(self) -> Result<(), Error> {
        self.tx.commit().await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl OperationStore for PostgresTransaction {
    type Error = Error;

    async fn get_operations_by_did(
        &self,
        did: &CanonicalPrismDid,
    ) -> Result<Vec<(OperationMetadata, SignedAtalaOperation)>, Self::Error> {
        let suffix_bytes = did.suffix().as_bytes();
        let result = entity::raw_operation::Entity::find()
            .filter(entity::raw_operation::Column::Did.eq(suffix_bytes))
            .all(&self.tx)
            .await?;

        result
            .into_iter()
            .map(|model| {
                let metadata = OperationMetadata {
                    block_metadata: BlockMetadata {
                        slot_number: model.slot as u64,
                        block_number: model.block_number as u64,
                        cbt: model.cbt,
                        absn: model.absn as u32,
                    },
                    osn: model.osn as u32,
                };
                SignedAtalaOperation::decode(model.signed_operation_data.as_slice())
                    .map(|op| (metadata, op))
                    .map_err(|e| Error::ProtobufDecode {
                        source: e,
                        backtrace: Backtrace::capture(),
                    })
            })
            .collect()
    }

    async fn insert(
        &self,
        signed_operation: SignedAtalaOperation,
        metadata: OperationMetadata,
    ) -> Result<(), Self::Error> {
        let did = get_did_from_signed_operation(&signed_operation)?;
        let active_model = entity::raw_operation::ActiveModel {
            did: did.suffix.to_vec().into_active_value(),
            signed_operation_data: signed_operation.encode_to_vec().into_active_value(),
            slot: (metadata.block_metadata.slot_number as i64).into_active_value(),
            block_number: (metadata.block_metadata.block_number as i64).into_active_value(),
            cbt: metadata.block_metadata.cbt.into_active_value(),
            absn: (metadata.block_metadata.absn as i32).into_active_value(),
            osn: (metadata.osn as i32).into_active_value(),
        };
        entity::raw_operation::Entity::insert(active_model)
            .on_conflict(
                OnConflict::columns([
                    entity::raw_operation::Column::Did,
                    entity::raw_operation::Column::BlockNumber,
                    entity::raw_operation::Column::Absn,
                    entity::raw_operation::Column::Osn,
                ])
                .update_columns([entity::raw_operation::Column::SignedOperationData])
                .to_owned(),
            )
            .exec(&self.tx)
            .await?;
        Ok(())
    }

    async fn get_all_dids(&self) -> Result<Vec<CanonicalPrismDid>, Self::Error> {
        let result = entity::raw_operation::Entity::find()
            .select_only()
            .column(entity::raw_operation::Column::Did)
            .column_as(entity::raw_operation::Column::BlockNumber.max(), "latest_block")
            .group_by(entity::raw_operation::Column::Did)
            .order_by_desc(Expr::col(Alias::new("latest_block")))
            .into_model::<DidProjection>()
            .all(&self.tx)
            .await?;
        result
            .into_iter()
            .map(|model| {
                let suffix = HexStr::from(model.did);
                CanonicalPrismDid::from_suffix(suffix).map_err(|e| DidError::from(e).into())
            })
            .collect()
    }
}

#[async_trait::async_trait]
impl DltCursorStore for PostgresTransaction {
    type Error = Error;

    async fn get_cursor(&self) -> Result<Option<DltCursor>, Self::Error> {
        let result = entity::dlt_cursor::Entity::find()
            .all(&self.tx)
            .await?
            .into_iter()
            .next()
            .map(|model| DltCursor {
                slot: model.slot as u64,
                block_hash: model.block_hash,
            });
        Ok(result)
    }

    async fn set_cursor(&self, cursor: DltCursor) -> Result<(), Self::Error> {
        let active_model = entity::dlt_cursor::ActiveModel {
            slot: (cursor.slot as i64).into_active_value(),
            block_hash: cursor.block_hash.into_active_value(),
        };
        let cursors = entity::dlt_cursor::Entity::find().all(&self.tx).await?;
        for c in cursors {
            c.delete(&self.tx).await?;
        }
        entity::dlt_cursor::Entity::insert(active_model)
            .on_conflict(
                OnConflict::columns([entity::dlt_cursor::Column::Slot, entity::dlt_cursor::Column::BlockHash])
                    .do_nothing()
                    .to_owned(),
            )
            .exec(&self.tx)
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl DltCursorStore for PostgresDb {
    type Error = Error;

    async fn get_cursor(&self) -> Result<Option<DltCursor>, Self::Error> {
        let tx = self.begin().await?;
        let result = tx.get_cursor().await?;
        tx.commit().await?;
        Ok(result)
    }

    async fn set_cursor(&self, cursor: DltCursor) -> Result<(), Self::Error> {
        let tx = self.begin().await?;
        tx.set_cursor(cursor).await?;
        tx.commit().await?;
        Ok(())
    }
}

#[derive(FromQueryResult)]
struct DidProjection {
    did: Vec<u8>,
}
