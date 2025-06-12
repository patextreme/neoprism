use identus_did_prism::did::operation::get_did_from_signed_operation;
use identus_did_prism::dlt::{BlockMetadata, DltCursor, OperationMetadata};
use identus_did_prism::prelude::*;
use identus_did_prism::proto::SignedPrismOperation;
use identus_did_prism::utils::paging::Paginated;
use identus_did_prism_indexer::repo::{DltCursorRepo, OperationRepo};
use lazybe::db::DbOps;
use lazybe::db::postgres::PostgresDbCtx;
use lazybe::filter::Filter;
use lazybe::page::PaginationInput;
use lazybe::sort::Sort;
use sqlx::PgPool;

use crate::{Error, entity};

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

    async fn get_all_dids(&self, page: u32, page_size: u32) -> Result<Paginated<CanonicalPrismDid>, Self::Error> {
        let mut tx = self.pool.begin().await?;
        let did_page = self
            .db_ctx
            .list::<entity::DidStats>(
                &mut tx,
                Filter::empty(),
                Sort::new([
                    entity::DidStatsSort::last_slot().desc(),
                    entity::DidStatsSort::did().asc(),
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
            current_page: did_page.page,
            page_size: did_page.page_size,
            total_items: did_page.total_records,
        })
    }

    async fn get_operations_by_did(
        &self,
        did: &CanonicalPrismDid,
    ) -> Result<Vec<(OperationMetadata, SignedPrismOperation)>, Self::Error> {
        let suffix_bytes = did.suffix().to_vec();
        let mut tx = self.pool.begin().await?;
        let result = self
            .db_ctx
            .list::<entity::RawOperation>(
                &mut tx,
                Filter::all([entity::RawOperationFilter::did().eq(suffix_bytes.into())]),
                Sort::empty(),
                None,
            )
            .await?
            .data
            .into_iter()
            .map(|model| {
                let metadata = OperationMetadata {
                    block_metadata: BlockMetadata {
                        slot_number: model.slot.try_into().expect("slot value does not fit in u64"),
                        block_number: model
                            .block_number
                            .try_into()
                            .expect("block_number value does not fit in u64"),
                        cbt: model.cbt,
                        absn: model.absn.try_into().expect("absn value does not fit in u32"),
                    },
                    osn: model.osn.try_into().expect("osn value does not fit in u32"),
                };
                SignedPrismOperation::decode(model.signed_operation_data.as_slice())
                    .map(|op| (metadata, op))
                    .map_err(|e| Error::ProtobufDecode {
                        source: e,
                        target_type: std::any::type_name::<SignedPrismOperation>(),
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        tx.commit().await?;
        Ok(result)
    }

    async fn insert_operations(
        &self,
        operations: Vec<(OperationMetadata, SignedPrismOperation)>,
    ) -> Result<(), Self::Error> {
        let mut tx = self.pool.begin().await?;
        for (metadata, signed_operation) in operations {
            let did = get_did_from_signed_operation(&signed_operation)
                .map_err(|e| Error::DidIndexFromSignedPrismOperation { source: e })?;

            let create_op = entity::CreateRawOperation {
                did: did.suffix.to_vec().into(),
                signed_operation_data: signed_operation.encode_to_vec(),
                slot: metadata
                    .block_metadata
                    .slot_number
                    .try_into()
                    .expect("slot_number does not fit in i64"),
                block_number: metadata
                    .block_metadata
                    .block_number
                    .try_into()
                    .expect("block_number does not fit in i64"),
                cbt: metadata.block_metadata.cbt,
                absn: metadata
                    .block_metadata
                    .absn
                    .try_into()
                    .expect("absn does not fit in i32"),
                osn: metadata.osn.try_into().expect("osn does not fit in i32"),
            };
            self.db_ctx.create::<entity::RawOperation>(&mut tx, create_op).await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl DltCursorRepo for PostgresDb {
    type Error = Error;

    async fn get_cursor(&self) -> Result<Option<DltCursor>, Self::Error> {
        let mut tx = self.pool.begin().await?;
        let result = self
            .db_ctx
            .list::<entity::DltCursor>(&mut tx, Filter::empty(), Sort::empty(), None)
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
            .list::<entity::DltCursor>(&mut tx, Filter::empty(), Sort::empty(), None)
            .await?
            .data;
        for c in cursors {
            self.db_ctx.delete::<entity::DltCursor>(&mut tx, c.id).await?;
        }
        self.db_ctx
            .create::<entity::DltCursor>(
                &mut tx,
                entity::CreateDltCursor {
                    slot: cursor.slot as i64,
                    block_hash: cursor.block_hash,
                },
            )
            .await?;
        tx.commit().await?;
        Ok(())
    }
}
