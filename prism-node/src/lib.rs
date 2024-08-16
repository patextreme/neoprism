#![allow(non_snake_case)]
#![feature(error_reporter)]

use app::service::DidService;
use clap::Parser;
use cli::CliArgs;
use prism_core::dlt::cardano::{NetworkIdentifier, OuraN2NSource};
use prism_core::dlt::DltCursor;
use prism_migration::run_migrations;
use prism_storage::PostgresDb;
use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::{Build, Rocket};

use crate::app::worker::DltSyncWorker;
use crate::http::routes;

mod app;
mod cli;
mod http;

struct AppState {
    did_service: DidService,
    cursor_rx: Option<tokio::sync::watch::Receiver<Option<DltCursor>>>,
    network: Option<NetworkIdentifier>,
}

pub fn build_rocket() -> Rocket<Build> {
    env_logger::init();

    let cli = CliArgs::parse();

    rocket::custom(cli.rocket_config())
        .manage(cli)
        .attach(init_database())
        .attach(init_state())
        .attach(init_endpoints())
}

fn init_database() -> AdHoc {
    AdHoc::on_ignite("Database Setup", |rocket| async move {
        let cli = rocket.state::<CliArgs>().expect("No CLI arguments provided");
        if cli.skip_migration {
            log::info!("Skipping database migrations");
        } else {
            log::info!("Applying database migrations");
            run_migrations(&cli.db).await.expect("Failed to apply migrations");
            log::info!("Applied database migrations successfully");
        }
        let db = PostgresDb::connect(&cli.db, false)
            .await
            .expect("Unable to connect to database");
        rocket.manage(db)
    })
}

fn init_state() -> AdHoc {
    AdHoc::on_ignite("AppState Setup", |rocket| async move {
        let cli = rocket.state::<CliArgs>().expect("No CLI arguments provided");
        let db = rocket.state::<PostgresDb>().expect("No PostgresDb provided");
        let did_service = DidService::new(db);

        let mut cursor_rx = None;
        let mut network = None;
        if let Some(address) = &cli.cardano {
            let network_identifier = cli.network.to_owned().unwrap_or(NetworkIdentifier::Mainnet);

            log::info!(
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
        rocket.manage(state)
    })
}

fn init_endpoints() -> AdHoc {
    AdHoc::on_ignite("Endpoints Setup", |rocket| async move {
        let cli = rocket.state::<CliArgs>().expect("No CLI arguments provided");
        let file_server = FileServer::from(cli.assets.clone());
        rocket.mount("/assets", file_server).mount(
            "/",
            rocket::routes!(routes::explorer, routes::hx::rpc, routes::index, routes::resolver,),
        )
    })
}
