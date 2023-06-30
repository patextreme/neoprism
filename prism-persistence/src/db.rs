use crate::migration::Migrator;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;

#[derive(Debug, Clone)]
pub struct PrismDB {
    db: DatabaseConnection,
}

impl PrismDB {
    pub async fn from_url(db_url: &str) -> Result<Self, DbErr> {
        let options = ConnectOptions::new(db_url.to_string());
        let db: DatabaseConnection = Database::connect(options).await?;
        Ok(Self { db })
    }

    pub async fn migrate(&self) -> Result<(), DbErr> {
        Migrator::up(&self.db, None).await?;
        Ok(())
    }
}

#[cfg(feature = "sqlite")]
pub mod sqlite {
    use super::PrismDB;
    use crate::{
        entity::{dlt_cursor, raw_operation},
        util::conv::Conv,
    };
    use prism_core::{
        dlt::OperationMetadata,
        prelude::CanonicalPrismDid,
        proto::SignedAtalaOperation,
        store::{CursorStoreError, DltCursor, DltCursorStore, OperationStore, OperationStoreError},
    };
    use sea_orm::*;

    #[async_trait::async_trait]
    impl OperationStore for PrismDB {
        async fn get_by_did(
            &mut self,
            did: &CanonicalPrismDid,
        ) -> Result<Vec<(OperationMetadata, SignedAtalaOperation)>, OperationStoreError> {
            let did_str = did.to_string();
            let operations = raw_operation::Entity::find()
                .filter(raw_operation::Column::Did.eq(did_str))
                .all(&self.db)
                .await
                .map_err(|e| OperationStoreError::StorageBackendError(e.into()))?;
            let mut result = Vec::with_capacity(operations.len());
            for operation in operations {
                let parsed: (OperationMetadata, SignedAtalaOperation) = operation
                    .try_into()
                    .map_err(OperationStoreError::StorageEncodingError)?;
                result.push(parsed);
            }
            Ok(result)
        }

        async fn insert(
            &mut self,
            signed_operation: SignedAtalaOperation,
            metadata: OperationMetadata,
        ) -> Result<(), OperationStoreError> {
            let operation: raw_operation::ActiveModel = (metadata, signed_operation)
                .try_into()
                .map_err(OperationStoreError::StorageEncodingError)?;
            operation.insert(&self.db).await.map_err(Conv)?;
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl DltCursorStore for PrismDB {
        async fn get_cursor(&mut self) -> Result<Option<DltCursor>, CursorStoreError> {
            let result = dlt_cursor::Entity::find()
                .order_by_desc(dlt_cursor::Column::Slot)
                .one(&self.db)
                .await
                .map_err(|e| CursorStoreError::StorageBackendError(e.into()))?
                .map(|i| DltCursor::try_from(i).map_err(CursorStoreError::StorageEncodingError));
            result.map_or(Ok(None), |v| v.map(Some))
        }

        async fn set_cursor(&mut self, cursor: DltCursor) -> Result<(), CursorStoreError> {
            let slot = <i32 as TryFrom<u64>>::try_from(cursor.slot)
                .map_err(|e| CursorStoreError::StorageEncodingError(e.into()))?;

            let txn = self.db.begin().await.map_err(Conv)?;

            dlt_cursor::Entity::delete_many()
                .exec(&txn)
                .await
                .map_err(Conv)?;

            dlt_cursor::ActiveModel {
                slot: ActiveValue::Set(slot),
                block_hash: ActiveValue::Set(cursor.block_hash),
            }
            .insert(&txn)
            .await
            .map_err(Conv)?;

            Ok(txn.commit().await.map_err(Conv)?)
        }
    }
}
