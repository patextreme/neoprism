use neoprism_node::run_command;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    run_command().await?;
    Ok(())
}
