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
    TestAudio(TestAudio),
}

#[derive(Debug, Clone, Parser)]
pub struct AllowRemote {
    pub addr: NodeId,
}

#[derive(Debug, Clone, Parser)]
pub struct TestAudio {
    #[clap(default_value = "rickroll")]
    pub source: AudioSource,
}
