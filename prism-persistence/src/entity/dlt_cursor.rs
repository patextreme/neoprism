//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "dlt_cursor")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub slot: i32,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))")]
    pub block_hash: Vec<u8>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
