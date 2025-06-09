use std::net::Ipv4Addr;
use std::path::PathBuf;

use clap::Parser;
use identus_did_prism::dlt::NetworkIdentifier;

#[derive(Parser)]
pub struct CliArgs {
    /// Database URL (e.g. postgres://user:pass@host:5432/db)
    #[arg(long, value_name = "DB_URL")]
    pub db: String,
    /// Skip database migration on Node startup
    #[arg(long)]
    pub skip_migration: bool,
    /// Address of the Cardano node to consume events from.
    /// If not provided, the Node will not sync events from the Cardano node.
    /// (e.g. backbone.mainnet.cardanofoundation.org:3001)
    #[arg(long, value_name = "CARDANO_ADDR")]
    pub cardano: Option<String>,
    /// A Cardano network to connect to.
    /// This option must correlate with the network of the node address provided.
    #[arg(long, value_name = "CARDANO_NETWORK", default_value_t = NetworkIdentifier::Mainnet, value_parser = parser::parse_network_identifier)]
    pub network: NetworkIdentifier,
    /// Node HTTP server binding address
    #[arg(long, default_value = "0.0.0.0")]
    pub address: Ipv4Addr,
    /// Node HTTP server listening port
    #[arg(short, long, default_value_t = 8080)]
    pub port: u16,
    /// The directory containing the web-ui assets (CSS, Javascripts)
    #[arg(long, default_value = "./indexer-node/assets")]
    pub assets: PathBuf,
}

mod parser {
    use std::str::FromStr;

    use identus_did_prism::dlt::NetworkIdentifier;

    pub fn parse_network_identifier(s: &str) -> Result<NetworkIdentifier, String> {
        let values = NetworkIdentifier::variants()
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        NetworkIdentifier::from_str(s).map_err(|_| format!("Invalid network {s}. Possible values: [{values}]"))
    }
}
