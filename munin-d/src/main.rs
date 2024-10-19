#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = munin_server::Config::get_or_create()?;
    let (_send, recv) = tokio::sync::mpsc::unbounded_channel();
    munin_server::run(config, recv).await?;
    Ok(())
}
