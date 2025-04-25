#![allow(non_snake_case)]
#![feature(error_reporter)]

use app::service::DidService;
use axum::Router;
use clap::Parser;
use cli::CliArgs;
use prism_core::dlt::DltCursor;
use prism_core::dlt::cardano::{NetworkIdentifier, OuraN2NSource};
use prism_storage::PostgresDb;

use crate::app::worker::DltSyncWorker;

mod app;
mod cli;
mod http;

#[derive(Clone)]
struct AppState {
    did_service: DidService,
    cursor_rx: Option<tokio::sync::watch::Receiver<Option<DltCursor>>>,
    network: Option<NetworkIdentifier>,
}

pub async fn start_server() -> anyhow::Result<()> {
    let cli = CliArgs::parse();

    let db = PostgresDb::connect(&cli.db)
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
    let mut cursor_rx = None;
    let mut network = None;
    if let Some(address) = &cli.cardano {
        let network_identifier = cli.network.to_owned();

        tracing::info!(
            "Starting DLT sync worker on {} from cardano address {}",
            network_identifier,
            address
        );
        let source = OuraN2NSource::since_persisted_cursor_or_genesis(db.clone(), address, &network_identifier)
            .await
            .expect("Failed to create DLT source");

        cursor_rx = Some(source.cursor_receiver());
        network = Some(network_identifier);
        let sync_app = DltSyncWorker::new(db.clone(), source);
        tokio::spawn(sync_app.run());
    }
    let state = AppState {
        did_service,
        cursor_rx,
        network,
    };

    let app = Router::new().merge(http::route::api::api_router()).with_state(state);

    let bind_addr = format!("{}:{}", cli.address, cli.port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("Server is listening on {}", bind_addr);
    axum::serve(listener, app).await?;

    Ok(())
}
