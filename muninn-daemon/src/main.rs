mod args;
use args::Args;
mod shared;
use clap::Parser;
use muninn_proto::AudioSource;
use shared::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let mut config = Config::get_or_create()?;
    if let Some(subcommand) = args.subcommand {
        match subcommand {
            args::Subcommand::AllowRemote(allow_remote) => {
                config.allowed_nodes.insert(allow_remote.addr);
                config.save()?;
            }
        }
    } else {
        run_daemon(config).await?;
    }
    Ok(())
}
