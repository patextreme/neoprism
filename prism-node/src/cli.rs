use clap::{Args, Parser, Subcommand};
use std::net::Ipv4Addr;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run database migration
    Migrate(MigrateArgs),
    /// Run PRISM node server
    Server(ServerArgs),
}

#[derive(Args)]
pub struct MigrateArgs {
    #[clap(flatten)]
    pub db_args: DbArgs,
}

#[derive(Args)]
pub struct ServerArgs {
    #[clap(flatten)]
    pub db_args: DbArgs,
    #[clap(flatten)]
    pub cardano_args: CaardanoArgs,
    /// PRISM node HTTP server bind address
    #[arg(long, default_value = "0.0.0.0", value_name = "ADDR")]
    pub bind: Ipv4Addr,
    /// PRISM node HTTP server listening port
    #[arg(short, long, default_value_t = 8080)]
    pub port: u16,
}

#[derive(Args)]
pub struct DbArgs {
    /// Database URL (e.g. sqlite://mydata.db)
    #[arg(long = "db", value_name = "DB_URL")]
    pub db_url: String,
}

#[derive(Args)]
pub struct CaardanoArgs {
    /// Address of the Cardano node to consume events from (e.g. relays-new.cardano-mainnet.iohk.io:3001)
    #[arg(long = "cardano_addr", value_name = "CARDANO_ADDR")]
    pub address: String,
}
