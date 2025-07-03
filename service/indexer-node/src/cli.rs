use std::net::Ipv4Addr;
use std::path::PathBuf;

use clap::Parser;
use identus_did_prism_indexer::dlt::NetworkIdentifier;

#[derive(Parser)]
pub struct CliArgs {
    /// Database URL (e.g. postgres://user:pass@host:5432/db)
    #[arg(long, env = "NPRISM_DB_URL")]
    pub db_url: String,
    /// Skip database migration on Node startup
    #[arg(long, env = "NPRISM_SKIP_MIGRATION")]
    pub skip_migration: bool,
    /// A Cardano network to connect.
    #[arg(long, short = 'n', env = "NPRISM_CARDANO_NETWORK", default_value_t = NetworkIdentifier::Mainnet, value_parser = parser::parse_network_identifier)]
    pub cardano_network: NetworkIdentifier,
    /// Address of the Cardano node to consume events from.
    /// If provided, it will sync events from the Cardano relay node.
    /// (e.g. backbone.mainnet.cardanofoundation.org:3001)
    #[arg(long, env = "NPRISM_CARDANO_ADDR", group = "dlt-source")]
    pub cardano_addr: Option<String>,
    /// DB-Sync url.
    /// If provided, it will sync events from the DB sync.
    /// (e.g. postgres://user:pass@host:5432/db)
    #[arg(long, env = "NPRISM_DBSYNC_URL", group = "dlt-source")]
    pub dbsync_url: Option<String>,
    /// Node HTTP server binding address
    #[arg(long, env = "NPRISM_ADDRESS", default_value = "0.0.0.0")]
    pub address: Ipv4Addr,
    /// Node HTTP server listening port
    #[arg(long, short, env = "NPRISM_PORT", default_value_t = 8080)]
    pub port: u16,
    /// The directory containing the web-ui assets (CSS, Javascripts)
    #[arg(long, env = "NPRISM_ASSETS_PATH", default_value = "./service/indexer-node/assets")]
    pub assets_path: PathBuf,
}

mod parser {
    use std::str::FromStr;

    use identus_did_prism_indexer::dlt::NetworkIdentifier;

    pub fn parse_network_identifier(s: &str) -> Result<NetworkIdentifier, String> {
        let values = NetworkIdentifier::variants()
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        NetworkIdentifier::from_str(s).map_err(|_| format!("Invalid network {s}. Possible values: [{values}]"))
    }
}
