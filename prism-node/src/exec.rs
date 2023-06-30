use crate::{
    app::PrismNodeApp,
    cli::{Cli, Commands, DbArgs, MigrateArgs, ServerArgs},
};
use prism_core::dlt::cardano::{NetworkIdentifier, OuraN2NSource};
use prism_persistence::db::PrismDB;

pub async fn execute_command(cli: &Cli) {
    match &cli.command {
        Commands::Migrate(args) => execute_migration(args).await,
        Commands::Server(args) => execute_server(args).await,
    }
}

async fn execute_migration(args: &MigrateArgs) {
    do_migrate(&args.db_args).await;
}

async fn execute_server(args: &ServerArgs) {
    let cardano_addr = &args.cardano_args.address;
    log::info!(
        "Using cardano node address {} as data source.",
        cardano_addr
    );

    // TODO: support custom network
    let store = do_migrate(&args.db_args).await;
    let source = OuraN2NSource::new_since_persisted_cursor(
        store.clone(),
        cardano_addr,
        &NetworkIdentifier::Mainnet,
    )
    .await
    .expect("Unable to create a OuraN2NSource");
    let node_app = PrismNodeApp::new(store, source);
    node_app.run().await;
}

async fn do_migrate(db_args: &DbArgs) -> PrismDB {
    log::info!("Executing database migration");
    let db_url = &db_args.db_url;
    let prism_db = PrismDB::from_url(db_url).await.unwrap();
    prism_db.migrate().await.unwrap();
    log::info!("Applied migration successfully");
    prism_db
}
