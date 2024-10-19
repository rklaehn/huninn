use clap::Parser;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(Debug, Clone, Parser)]
pub enum Subcommand {
    Install,
    Uninstall,
    QueryConfig,
    Pause,
    Resume,
    Start(Start),
    Stop,
}

#[derive(Debug, Clone, Parser)]
pub struct Start {}
