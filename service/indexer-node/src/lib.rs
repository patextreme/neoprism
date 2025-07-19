#![allow(non_snake_case)]
#![feature(error_reporter)]

use app::service::DidService;
use clap::Parser;
use cli::CliArgs;
use identus_did_prism::dlt::DltCursor;
use identus_did_prism_indexer::dlt::NetworkIdentifier;
use identus_did_prism_indexer::dlt::dbsync::DbSyncSource;
use identus_did_prism_indexer::dlt::oura::OuraN2NSource;
use indexer_storage::PostgresDb;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::app::worker::{DltIndexWorker, DltSyncWorker};

mod app;
mod cli;
mod http;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone)]
struct AppState {
    did_service: DidService,
    cursor_rx: Option<tokio::sync::watch::Receiver<Option<DltCursor>>>,
    network: NetworkIdentifier,
}

pub async fn start_server() -> anyhow::Result<()> {
    let cli = CliArgs::parse();

    let db = PostgresDb::connect(&cli.db_url)
        .await
        .expect("Unable to connect to database");

    // init migrations
    if cli.skip_migration {
        tracing::info!("Skipping database migrations");
    } else {
        tracing::info!("Applying database migrations");
        db.migrate().await.expect("Failed to apply migrations");
        tracing::info!("Applied database migrations successfully");
    }

    // init state
    let did_service = DidService::new(&db);
    let network = cli.cardano_network;
    let cursor_rx = start_dlt_source(&cli, &network, &db).await;

    let state = AppState {
        did_service,
        cursor_rx,
        network,
    };

    // start server
    let layer = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .option_layer(Some(CorsLayer::permissive()).filter(|_| cli.cors_enabled));
    let router = http::router(&cli.assets_path).with_state(state).layer(layer);
    let bind_addr = format!("{}:{}", cli.address, cli.port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("Server is listening on {}", bind_addr);
    axum::serve(listener, router).await?;

    Ok(())
}

async fn start_dlt_source(
    cli: &CliArgs,
    network: &NetworkIdentifier,
    db: &PostgresDb,
) -> Option<tokio::sync::watch::Receiver<Option<DltCursor>>> {
    if let Some(address) = &cli.cardano_addr {
        tracing::info!(
            "Starting DLT sync worker on {} from cardano address {}",
            network,
            address
        );
        let source = OuraN2NSource::since_persisted_cursor_or_genesis(db.clone(), address, network)
            .await
            .expect("Failed to create DLT source");

        let sync_worker = DltSyncWorker::new(db.clone(), source);
        let index_worker = DltIndexWorker::new(db.clone());
        let cursor_rx = sync_worker.sync_cursor();
        tokio::spawn(sync_worker.run());
        tokio::spawn(index_worker.run());

        Some(cursor_rx)
    } else if let Some(dbsync_url) = cli.dbsync_url.as_ref() {
        tracing::info!("Starting DLT sync worker on {} from cardano dbsync", network);
        let source = DbSyncSource::since_persisted_cursor(db.clone(), dbsync_url)
            .await
            .expect("Failed to create DLT source");

        let sync_worker = DltSyncWorker::new(db.clone(), source);
        let index_worker = DltIndexWorker::new(db.clone());
        let cursor_rx = sync_worker.sync_cursor();
        tokio::spawn(sync_worker.run());
        tokio::spawn(index_worker.run());

        Some(cursor_rx)
    } else {
        None
    }
}
