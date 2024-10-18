use clap::Parser;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(Debug, Clone, Parser)]
pub enum Subcommand {
    Install(Install),
    Uninstall(Uninstall),
    QueryConfig(QueryConfig),
    Pause(Pause),
    Resume(Resume),
    Start(Start),
    Stop(Stop),
}

#[derive(Debug, Clone, Parser)]
pub struct Install {}

#[derive(Debug, Clone, Parser)]
pub struct Uninstall {}

#[derive(Debug, Clone, Parser)]
pub struct QueryConfig {}

#[derive(Debug, Clone, Parser)]
pub struct Pause {}

#[derive(Debug, Clone, Parser)]
pub struct Resume {}

#[derive(Debug, Clone, Parser)]
pub struct Start {}

#[derive(Debug, Clone, Parser)]
pub struct Stop {}
