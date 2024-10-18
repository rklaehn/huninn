mod args;
mod shared;
use shared::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = Config::get_or_create()?;
    let (_send, recv) = tokio::sync::mpsc::unbounded_channel();
    run_daemon(config, recv).await?;
    Ok(())
}
