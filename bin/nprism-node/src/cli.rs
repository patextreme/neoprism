use std::net::Ipv4Addr;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use identus_did_prism_indexer::dlt::NetworkIdentifier;

#[derive(Parser)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start the node in indexer mode.
    Indexer(IndexerArgs),
    /// Start the node in submitter mode.
    Submitter(SubmitterArgs),
}

#[derive(Args)]
pub struct IndexerArgs {
    #[clap(flatten)]
    pub server: ServerArgs,
    #[clap(flatten)]
    pub db: DbArgs,
    #[clap(flatten)]
    pub dlt_source: DltSourceArgs,
}

#[derive(Args)]
pub struct SubmitterArgs {
    #[clap(flatten)]
    pub server: ServerArgs,
    #[clap(flatten)]
    pub db: DbArgs,
}

#[derive(Args)]
pub struct ServerArgs {
    /// Node HTTP server binding address
    #[arg(long, env = "NPRISM_ADDRESS", default_value = "0.0.0.0")]
    pub address: Ipv4Addr,
    /// Node HTTP server listening port
    #[arg(long, short, env = "NPRISM_PORT", default_value_t = 8080)]
    pub port: u16,
    /// The directory containing the web-ui assets (CSS, Javascripts)
    #[arg(long, env = "NPRISM_ASSETS_PATH", default_value = "./bin/nprism-node/assets")]
    pub assets_path: PathBuf,
    /// Enable permissive CORS (https://docs.rs/tower-http/latest/tower_http/cors/struct.CorsLayer.html#method.permissive)
    #[arg(long, env = "NPRISM_CORS_ENABLED")]
    pub cors_enabled: bool,
}

#[derive(Args)]
pub struct DbArgs {
    /// Database URL (e.g. postgres://user:pass@host:5432/db)
    #[arg(long, env = "NPRISM_DB_URL")]
    pub db_url: String,
    /// Skip database migration on Node startup
    #[arg(long, env = "NPRISM_SKIP_MIGRATION")]
    pub skip_migration: bool,
}

#[derive(Args)]
pub struct DltSourceArgs {
    /// A Cardano network the node is syncing from.
    #[arg(long, env = "NPRISM_CARDANO_NETWORK")]
    pub cardano_network: NetworkIdentifierOptions,
    /// Address of the Cardano relay node to sync from.
    /// If provided, it will sync events from the Cardano relay node.
    /// (e.g. backbone.mainnet.cardanofoundation.org:3001)
    #[arg(long, env = "NPRISM_CARDANO_RELAY", group = "dlt-source")]
    pub cardano_relay: Option<String>,
    /// DB-Sync url.
    /// If provided, it will sync events from the DB sync.
    /// (e.g. postgres://user:pass@host:5432/db)
    #[arg(long, env = "NPRISM_CARDANO_DBSYNC_URL", group = "dlt-source")]
    pub cardano_dbsync_url: Option<String>,
    /// Number of confirmation blocks to wait before considering the block valid.
    #[arg(long, env = "NPRISM_CONFIRMATION_BLOCKS", default_value_t = 112)]
    pub confirmation_blocks: usize,
}

#[derive(Clone, ValueEnum)]
pub enum NetworkIdentifierOptions {
    Mainnet,
    Preprod,
    Preview,
}

impl From<NetworkIdentifierOptions> for NetworkIdentifier {
    fn from(value: NetworkIdentifierOptions) -> Self {
        match value {
            NetworkIdentifierOptions::Mainnet => NetworkIdentifier::Mainnet,
            NetworkIdentifierOptions::Preprod => NetworkIdentifier::Preprod,
            NetworkIdentifierOptions::Preview => NetworkIdentifier::Preview,
        }
    }
}
