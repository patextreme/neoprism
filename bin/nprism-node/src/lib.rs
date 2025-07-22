#![allow(non_snake_case)]
#![feature(error_reporter)]

use app::service::DidService;
use clap::Parser;
use cli::Cli;
use identus_did_prism::dlt::DltCursor;
use identus_did_prism_indexer::dlt::NetworkIdentifier;
use identus_did_prism_indexer::dlt::dbsync::DbSyncSource;
use identus_did_prism_indexer::dlt::oura::OuraN2NSource;
use indexer_storage::PostgresDb;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::app::worker::{DltIndexWorker, DltSyncWorker};
use crate::cli::{DbArgs, DltSourceArgs, IndexerArgs, ServerArgs};

mod app;
mod cli;
mod http;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone)]
struct AppState {
    did_service: DidService,
    dlt_source: Option<DltSourceState>,
}

#[derive(Clone)]
struct DltSourceState {
    cursor_rx: tokio::sync::watch::Receiver<Option<DltCursor>>,
    network: NetworkIdentifier,
}

pub async fn run_command() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        cli::Command::Indexer(args) => run_indexer_command(args).await?,
        cli::Command::Submitter(_) => todo!("implement"),
    };
    Ok(())
}

async fn run_indexer_command(args: IndexerArgs) -> anyhow::Result<()> {
    let db_args = &args.db;
    let server_args = &args.server;
    let dlt_args = &args.dlt_source;

    // init database
    let db = init_database(db_args).await;

    // init state
    let network = dlt_args.cardano_network.clone().into();
    let cursor_rx = init_dlt_source(&dlt_args, &network, &db).await;
    let app_state = AppState {
        did_service: DidService::new(&db),
        dlt_source: Some(DltSourceState { cursor_rx, network }),
    };

    run_server(app_state, server_args).await
}

async fn run_server(app_state: AppState, server_args: &ServerArgs) -> anyhow::Result<()> {
    let layer = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .option_layer(Some(CorsLayer::permissive()).filter(|_| server_args.cors_enabled));
    let router = http::router(&server_args.assets_path)
        .with_state(app_state)
        .layer(layer);
    let bind_addr = format!("{}:{}", server_args.address, server_args.port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("Server is listening on {}", bind_addr);
    axum::serve(listener, router).await?;
    Ok(())
}

async fn init_database(db_args: &DbArgs) -> PostgresDb {
    let db = PostgresDb::connect(&db_args.db_url)
        .await
        .expect("Unable to connect to database");

    if db_args.skip_migration {
        tracing::info!("Skipping database migrations");
    } else {
        tracing::info!("Applying database migrations");
        db.migrate().await.expect("Failed to apply migrations");
        tracing::info!("Applied database migrations successfully");
    }

    db
}

async fn init_dlt_source(
    dlt_args: &DltSourceArgs,
    network: &NetworkIdentifier,
    db: &PostgresDb,
) -> tokio::sync::watch::Receiver<Option<DltCursor>> {
    if let Some(address) = &dlt_args.cardano_relay {
        tracing::info!(
            "Starting DLT sync worker on {} from cardano address {}",
            network,
            address
        );
        let source = OuraN2NSource::since_persisted_cursor_or_genesis(
            db.clone(),
            address,
            network,
            dlt_args.confirmation_blocks,
        )
        .await
        .expect("Failed to create DLT source");

        let sync_worker = DltSyncWorker::new(db.clone(), source);
        let index_worker = DltIndexWorker::new(db.clone());
        let cursor_rx = sync_worker.sync_cursor();
        tokio::spawn(sync_worker.run());
        tokio::spawn(index_worker.run());
        cursor_rx
    } else if let Some(dbsync_url) = dlt_args.cardano_dbsync_url.as_ref() {
        tracing::info!("Starting DLT sync worker on {} from cardano dbsync", network);
        let source = DbSyncSource::since_persisted_cursor(db.clone(), dbsync_url, dlt_args.confirmation_blocks)
            .await
            .expect("Failed to create DLT source");

        let sync_worker = DltSyncWorker::new(db.clone(), source);
        let index_worker = DltIndexWorker::new(db.clone());
        let cursor_rx = sync_worker.sync_cursor();
        tokio::spawn(sync_worker.run());
        tokio::spawn(index_worker.run());
        cursor_rx
    } else {
        panic!("DLT source is not configured.")
    }
}
