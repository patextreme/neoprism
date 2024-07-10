use std::net::Ipv4Addr;
use std::path::PathBuf;

use clap::Parser;
use rocket::Config;

#[derive(Parser)]
pub struct CliArgs {
    /// Database URL (e.g. postgres://user:pass@host:5432/db)
    #[arg(long, value_name = "DB_URL")]
    pub db: String,
    /// Skip database migration on Node startup
    #[arg(long, default_value_t = false)]
    pub skip_migration: bool,
    /// Address of the Cardano node to consume events from.
    /// If not provided, the Node will not sync events from the Cardano node.
    /// (e.g. relays-new.cardano-mainnet.iohk.io:3001)
    #[arg(long, value_name = "CARDANO_ADDR")]
    pub cardano: Option<String>,
    /// Node HTTP server binding address
    #[arg(long, default_value = "0.0.0.0", value_name = "ADDR")]
    pub address: Ipv4Addr,
    /// Node HTTP server listening port
    #[arg(short, long, default_value_t = 8080)]
    pub port: u16,
    /// Path of the directory containing the Node assets
    #[arg(long, default_value = "./prism-node/assets")]
    pub assets: PathBuf,
}

impl CliArgs {
    pub fn rocket_config(&self) -> Config {
        let config = Config::default();
        Config {
            address: self.address.into(),
            port: self.port,
            ..config
        }
    }
}
