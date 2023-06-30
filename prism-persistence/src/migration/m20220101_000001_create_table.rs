use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(RawOperation::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(RawOperation::Did).binary_len(32).not_null())
                    .col(
                        ColumnDef::new(RawOperation::SignedOperationData)
                            .binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(RawOperation::Slot).big_integer().not_null())
                    .col(
                        ColumnDef::new(RawOperation::BlockNumber)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(RawOperation::Cbt).date_time().not_null())
                    .col(ColumnDef::new(RawOperation::Absn).integer().not_null())
                    .col(ColumnDef::new(RawOperation::Osn).integer().not_null())
                    .primary_key(
                        Index::create()
                            .name("raw_operation_did_time_idx")
                            .col(RawOperation::Did)
                            .col(RawOperation::BlockNumber)
                            .col(RawOperation::Absn)
                            .col(RawOperation::Osn)
                            .primary(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(DltCursor::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DltCursor::Slot)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(DltCursor::BlockHash).binary().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("raw_operation_did_idx")
                    .table(RawOperation::Table)
                    .col(RawOperation::Did)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("raw_operation_cbt_idx")
                    .table(RawOperation::Table)
                    .col(RawOperation::Cbt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(Iden)]
enum RawOperation {
    Table,
    Did,
    SignedOperationData,
    Cbt,
    Absn,
    Osn,
    BlockNumber,
    Slot,
}

#[derive(Iden)]
enum DltCursor {
    Table,
    Slot,
    BlockHash,
}
