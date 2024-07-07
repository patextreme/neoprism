use clap::Parser;

mod app;
mod cli;
mod exec;

#[tokio::main]
async fn main() {
    env_logger::init();

    let cli = cli::Cli::parse();
    exec::execute_command(&cli)
        .await
        .expect("Program exited with an error");
}
