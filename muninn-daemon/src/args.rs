use clap::Parser;
use iroh_net::NodeId;
use muninn_proto::AudioSource;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub subcommand: Option<Subcommand>,
}

#[derive(Debug, Clone, Parser)]
pub enum Subcommand {
    AllowRemote(AllowRemote),
}

#[derive(Debug, Clone, Parser)]
pub struct AllowRemote {
    pub addr: NodeId,
}
