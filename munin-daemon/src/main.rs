mod args;
mod shared;
use shared::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let (_send, recv) = tokio::sync::mpsc::unbounded_channel();
    run_daemon(recv).await?;
    Ok(())
}
