pub use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectOptions, Database};

mod m20240701_000001_create_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20240701_000001_create_table::Migration)]
    }
}

pub async fn run_migrations(db_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let connect_options = ConnectOptions::new(db_url).set_schema_search_path("public").to_owned();
    let db = &Database::connect(connect_options).await?;
    Migrator::up(db, None).await?;
    Ok(())
}
