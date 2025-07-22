#![allow(non_snake_case)]
#![feature(error_reporter)]

use app::service::DidService;
use axum::Router;
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
use crate::cli::{DbArgs, DltSourceArgs, IndexerArgs};

mod app;
mod cli;
mod http;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone)]
struct IndexerState {
    did_service: DidService,
    cursor_rx: Option<tokio::sync::watch::Receiver<Option<DltCursor>>>,
    network: NetworkIdentifier,
}

pub async fn start_server() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let db_args = match &cli.command {
        cli::Command::Indexer(args) => &args.db,
        cli::Command::Submitter(args) => &args.db,
    };

    let server_args = match &cli.command {
        cli::Command::Indexer(args) => &args.server,
        cli::Command::Submitter(args) => &args.server,
    };

    let dlt_args = match &cli.command {
        cli::Command::Indexer(args) => Some(&args.dlt_source),
        cli::Command::Submitter(_) => None,
    };

    // init database
    let db = init_database(db_args).await;

    // init indexer
    let indexer_router = if let Some(dlt_args) = dlt_args {
        let did_service = DidService::new(&db);
        let network = dlt_args.cardano_network.clone().into();
        let cursor_rx = init_dlt_source(&dlt_args, &network, &db).await;
        let indexer_state = IndexerState {
            did_service,
            cursor_rx,
            network,
        };
        http::router(&server_args.assets_path).with_state(indexer_state)
    } else {
        Router::new()
    };

    // start server
    let layer = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .option_layer(Some(CorsLayer::permissive()).filter(|_| server_args.cors_enabled));
    let router = Router::new().merge(indexer_router).layer(layer);
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
) -> Option<tokio::sync::watch::Receiver<Option<DltCursor>>> {
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
        Some(cursor_rx)
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
        Some(cursor_rx)
    } else {
        None
    }
}
