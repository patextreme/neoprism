use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RawOperation::Table)
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
                    .col(
                        ColumnDef::new(RawOperation::Cbt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(RawOperation::Absn).integer().not_null())
                    .col(ColumnDef::new(RawOperation::Osn).integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(RawOperation::Did)
                            .col(RawOperation::BlockNumber)
                            .col(RawOperation::Absn)
                            .col(RawOperation::Osn),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(DltCursor::Table)
                    .col(ColumnDef::new(DltCursor::Slot).big_integer().not_null())
                    .col(ColumnDef::new(DltCursor::BlockHash).binary().not_null())
                    .primary_key(
                        Index::create()
                            .col(DltCursor::Slot)
                            .col(DltCursor::BlockHash),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveIden)]
enum RawOperation {
    Table,
    Did,
    SignedOperationData,
    Slot,
    BlockNumber,
    Cbt,
    Absn,
    Osn,
}

#[derive(DeriveIden)]
enum DltCursor {
    Table,
    Slot,
    BlockHash,
}
