#![feature(error_generic_member_access)]

use lazybe::db::DbOps;
use lazybe::db::postgres::PostgresDbCtx;
use lazybe::filter::Filter;
use lazybe::page::PaginationInput;
use lazybe::sort::Sort;
use prism_core::did::Error as DidError;
use prism_core::did::error::DidSyntaxError;
use prism_core::did::operation::get_did_from_signed_operation;
use prism_core::dlt::{BlockMetadata, DltCursor, OperationMetadata};
use prism_core::prelude::*;
use prism_core::proto::SignedAtalaOperation;
use prism_core::repo::{DltCursorRepo, OperationRepo};
use prism_core::utils::paging::Paginated;
use sqlx::PgPool;

mod entity;
mod entity2;

#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum Error {
    #[from]
    #[display("database connection error")]
    Db { source: sqlx::Error },
    #[from]
    #[display("database migration error")]
    DbMigration { source: sqlx::migrate::MigrateError },
    #[display("unable to decode to protobuf message into type {target_type} from stored data")]
    ProtobufDecode {
        source: prost::DecodeError,
        target_type: &'static str,
    },
    #[from]
    #[display("cannot compute did index from SignedAtalaOperation")]
    DidIndexFromSignedAtalaOperation { source: DidError },
    #[from]
    #[display("cannot decode did from stored data")]
    DidDecode { source: DidSyntaxError },
}

#[derive(Debug, Clone)]
pub struct PostgresDb {
    pool: PgPool,
    db_ctx: PostgresDbCtx,
}

impl PostgresDb {
    pub async fn connect(db_url: &str) -> Result<Self, Error> {
        let pool = PgPool::connect(db_url).await?;
        Ok(Self {
            db_ctx: PostgresDbCtx,
            pool,
        })
    }

    pub async fn migrate(&self) -> Result<(), Error> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl OperationRepo for PostgresDb {
    type Error = Error;

    async fn get_operations_by_did(
        &self,
        did: &CanonicalPrismDid,
    ) -> Result<Vec<(OperationMetadata, SignedAtalaOperation)>, Self::Error> {
        let suffix_bytes = did.suffix().to_vec();
        let mut tx = self.pool.begin().await?;
        let result = self
            .db_ctx
            .list::<entity2::RawOperation>(
                &mut tx,
                Filter::all([entity2::RawOperationFilter::did().eq(suffix_bytes.into())]),
                Sort::empty(),
                None,
            )
            .await?
            .data
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
                        target_type: std::any::type_name::<SignedAtalaOperation>(),
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        tx.commit().await?;
        Ok(result)
    }

    async fn insert_operation(
        &self,
        signed_operation: SignedAtalaOperation,
        metadata: OperationMetadata,
    ) -> Result<(), Self::Error> {
        let did = get_did_from_signed_operation(&signed_operation)
            .map_err(|e| Error::DidIndexFromSignedAtalaOperation { source: e })?;
        let mut tx = self.pool.begin().await?;
        self.db_ctx
            .create::<entity2::RawOperation>(
                &mut tx,
                entity2::CreateRawOperation {
                    did: did.suffix.to_vec().into(),
                    signed_operation_data: signed_operation.encode_to_vec(),
                    slot: metadata.block_metadata.slot_number as i64,
                    block_number: metadata.block_metadata.block_number as i64,
                    cbt: metadata.block_metadata.cbt,
                    absn: metadata.block_metadata.absn as i32,
                    osn: metadata.osn as i32,
                },
            )
            .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn get_all_dids(&self, page: u32, page_size: u32) -> Result<Paginated<CanonicalPrismDid>, Self::Error> {
        let mut tx = self.pool.begin().await?;
        let did_page = self
            .db_ctx
            .list::<entity2::DidStats>(
                &mut tx,
                Filter::empty(),
                Sort::new([
                    entity2::DidStatsSort::last_slot().desc(),
                    entity2::DidStatsSort::did().asc(),
                ]),
                Some(PaginationInput { page, limit: page_size }),
            )
            .await?;
        tx.commit().await?;

        let items = did_page
            .data
            .into_iter()
            .map(|stats| stats.did.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Paginated {
            items,
            current_page: did_page.page.into(),
            page_size: did_page.page_size,
            total_items: did_page.total_records,
        })
    }
}

#[async_trait::async_trait]
impl DltCursorRepo for PostgresDb {
    type Error = Error;

    async fn get_cursor(&self) -> Result<Option<DltCursor>, Self::Error> {
        let mut tx = self.pool.begin().await?;
        let result = self
            .db_ctx
            .list::<entity2::DltCursor>(&mut tx, Filter::empty(), Sort::empty(), None)
            .await?
            .data
            .into_iter()
            .next()
            .map(|model| DltCursor {
                slot: model.slot as u64,
                block_hash: model.block_hash,
                cbt: None,
            });
        tx.commit().await?;
        Ok(result)
    }

    async fn set_cursor(&self, cursor: DltCursor) -> Result<(), Self::Error> {
        let mut tx = self.pool.begin().await?;
        let cursors = self
            .db_ctx
            .list::<entity2::DltCursor>(&mut tx, Filter::empty(), Sort::empty(), None)
            .await?
            .data;
        for c in cursors {
            self.db_ctx.delete::<entity2::DltCursor>(&mut tx, c.id).await?;
        }
        self.db_ctx
            .create::<entity2::DltCursor>(
                &mut tx,
                entity2::CreateDltCursor {
                    slot: cursor.slot as i64,
                    block_hash: cursor.block_hash,
                },
            )
            .await?;
        tx.commit().await?;
        Ok(())
    }
}

struct DidProjection {
    did: Vec<u8>,
}

struct CountProjection {
    count: Option<i64>,
}
