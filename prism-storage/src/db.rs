use super::table;
use prism_core::{
    dlt::OperationMetadata,
    prelude::CanonicalPrismDid,
    proto::SignedAtalaOperation,
    store::{
        get_did_from_signed_operation, CursorStoreError, DltCursor, DltCursorStore, OperationStore,
        OperationStoreError,
    },
    util::MessageExt,
};
use sea_query::{Expr, OnConflict, Query, SqliteQueryBuilder};
use sea_query_binder::SqlxBinder;
use sqlx::SqlitePool;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum PrismDBError {
    #[error("Error occur during database execution. {0}")]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Clone)]
pub struct PrismDB {
    pool: SqlitePool,
}

impl PrismDB {
    pub async fn in_memory() -> Result<Self, PrismDBError> {
        Self::connect_url("sqlite::memory:").await
    }

    pub async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        log::info!("Executing database migrations");
        let result = sqlx::migrate!("./migrations").run(&self.pool).await;
        log::info!("All database migrations applied successfully");
        result
    }
    pub async fn connect_path(path: &Path) -> Result<Self, PrismDBError> {
        let url = format!("sqlite://{}", path.to_string_lossy());
        Self::connect_url(&url).await
    }

    pub async fn connect_url(db_url: &str) -> Result<Self, PrismDBError> {
        log::info!("Connecting to database {}", db_url);
        let pool = SqlitePool::connect(db_url).await?;
        log::info!("Connect to database successfully");
        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl OperationStore for PrismDB {
    async fn get_by_did(
        &self,
        did: &CanonicalPrismDid,
    ) -> Result<Vec<(OperationMetadata, SignedAtalaOperation)>, OperationStoreError> {
        let did_suffix = did.suffix.as_bytes();
        let query = Query::select()
            .columns([
                table::RawOperation::Did,
                table::RawOperation::SignedOperationData,
                table::RawOperation::Slot,
                table::RawOperation::BlockNumber,
                table::RawOperation::Cbt,
                table::RawOperation::Absn,
                table::RawOperation::Osn,
            ])
            .from(table::RawOperation::Table)
            .and_where(Expr::col(table::RawOperation::Did).eq(did_suffix))
            .to_owned();

        let (query_str, values) = query.build_sqlx(SqliteQueryBuilder);
        let rows: Vec<table::RawOperationRow> = sqlx::query_as_with(&query_str, values)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| OperationStoreError::StorageBackendError(e.into()))?;

        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            let parsed: (OperationMetadata, SignedAtalaOperation) = row
                .try_into()
                .map_err(OperationStoreError::StorageEncodingError)?;
            result.push(parsed);
        }

        Ok(result)
    }

    async fn insert(
        &self,
        signed_operation: SignedAtalaOperation,
        metadata: OperationMetadata,
    ) -> Result<(), OperationStoreError> {
        let did = get_did_from_signed_operation(&signed_operation)?;
        let query = Query::insert()
            .columns([
                table::RawOperation::Did,
                table::RawOperation::SignedOperationData,
                table::RawOperation::Slot,
                table::RawOperation::BlockNumber,
                table::RawOperation::Cbt,
                table::RawOperation::Absn,
                table::RawOperation::Osn,
            ])
            .into_table(table::RawOperation::Table)
            .values([
                did.suffix.as_bytes().into(),
                signed_operation.encode_to_bytes()?.as_ref().into(),
                metadata.block_metadata.slot_number.into(),
                metadata.block_metadata.block_number.into(),
                metadata.block_metadata.cbt.into(),
                metadata.block_metadata.absn.into(),
                metadata.osn.into(),
            ])
            .map_err(|e| OperationStoreError::StorageBackendError(e.into()))?
            .on_conflict(
                OnConflict::columns([
                    table::RawOperation::Did,
                    table::RawOperation::BlockNumber,
                    table::RawOperation::Absn,
                    table::RawOperation::Osn,
                ])
                .update_columns([
                    table::RawOperation::Did,
                    table::RawOperation::BlockNumber,
                    table::RawOperation::Absn,
                    table::RawOperation::Osn,
                ])
                .to_owned(),
            )
            .to_owned();

        let (query_str, values) = query.build_sqlx(SqliteQueryBuilder);
        let _ = sqlx::query_with(&query_str, values)
            .execute(&self.pool)
            .await
            .map_err(|e| OperationStoreError::StorageBackendError(e.into()))?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl DltCursorStore for PrismDB {
    async fn get_cursor(&self) -> Result<Option<DltCursor>, CursorStoreError> {
        let query = Query::select()
            .columns([table::DltCursor::Slot, table::DltCursor::BlockHash])
            .from(table::DltCursor::Table)
            .to_owned();
        let (query_str, values) = query.build_sqlx(SqliteQueryBuilder);
        let result: Option<table::DltCursorRow> = sqlx::query_as_with(&query_str, values)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| CursorStoreError::StorageBackendError(e.into()))?;

        match result {
            Some(row) => {
                let parsed = DltCursor::try_from(row)
                    .map_err(|e| CursorStoreError::StorageEncodingError(e.into()))?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    async fn set_cursor(&self, cursor: DltCursor) -> Result<(), CursorStoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CursorStoreError::StorageBackendError(e.into()))?;

        // delete current cursor
        let query = Query::delete()
            .from_table(table::DltCursor::Table)
            .to_owned();
        let (query_str, values) = query.build_sqlx(SqliteQueryBuilder);
        let _ = sqlx::query_with(&query_str, values)
            .execute(&mut *tx)
            .await
            .map_err(|e| CursorStoreError::StorageBackendError(e.into()))?;

        // set new cursor
        let query = Query::insert()
            .into_table(table::DltCursor::Table)
            .columns([table::DltCursor::Slot, table::DltCursor::BlockHash])
            .values([cursor.slot.into(), cursor.block_hash.into()])
            .map_err(|e| CursorStoreError::StorageBackendError(e.into()))?
            .to_owned();
        let (query_str, values) = query.build_sqlx(SqliteQueryBuilder);
        let _ = sqlx::query_with(&query_str, values)
            .execute(&mut *tx)
            .await
            .map_err(|e| CursorStoreError::StorageBackendError(e.into()))?;

        tx.commit()
            .await
            .map_err(|e| CursorStoreError::StorageBackendError(e.into()))?;

        Ok(())
    }
}
