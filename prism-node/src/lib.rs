#![allow(non_snake_case)]

use app::service::DidService;
use clap::Parser;
use cli::CliArgs;
use prism_core::dlt::cardano::{NetworkIdentifier, OuraN2NSource};
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

pub fn build_rocket() -> Rocket<Build> {
    env_logger::init();

    let cli = CliArgs::parse();

    rocket::custom(cli.rocket_config())
        .manage(cli)
        .attach(init_database())
        .attach(init_services())
        .attach(init_workers())
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

fn init_services() -> AdHoc {
    AdHoc::on_ignite("Inject services", |rocket| async move {
        let db = rocket.state::<PostgresDb>().expect("No PostgresDb provided");
        let did_service = DidService::new(db);
        rocket.manage(did_service)
    })
}

fn init_workers() -> AdHoc {
    AdHoc::on_ignite("Inject background workers", |rocket| async move {
        let cli = rocket.state::<CliArgs>().expect("No CLI arguments provided");
        let db = rocket.state::<PostgresDb>().expect("No PostgresDb provided");

        if let Some(address) = &cli.cardano {
            log::info!("Starting DLT sync worker with cardano address {}", address);
            let source =
                OuraN2NSource::since_persisted_cursor_or_genesis(db.clone(), address, &NetworkIdentifier::Mainnet)
                    .await
                    .expect("Failed to create DLT source");

            let sync_app = DltSyncWorker::new(db.clone(), source);
            tokio::spawn(sync_app.run());
        }
        rocket
    })
}

fn init_endpoints() -> AdHoc {
    AdHoc::on_ignite("Inject endpoints", |rocket| async move {
        let cli = rocket.state::<CliArgs>().expect("No CLI arguments provided");
        let file_server = FileServer::from(cli.assets.clone());
        rocket.mount("/assets", file_server).mount(
            "/",
            rocket::routes!(routes::index, routes::resolver, routes::hx::resolve_did),
        )
    })
}
