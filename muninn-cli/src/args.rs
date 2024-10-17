use clap::Parser;
use iroh_net::NodeId;
use muninn_proto::AudioSource;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(Debug, Clone, Parser)]
pub enum Subcommand {
    AddNode(AddNode),
    RemoveNode(RemoveNode),
    ListNodes(ListNodes),

    ListTasks(ListTasks),
    KillTask(KillTask),
    SystemInfo(SystemInfo),
    PlayAudio(PlayAudio),
    Shutdown(Shutdown),
}

#[derive(Debug, Clone, Parser)]
pub struct AddNode {
    #[clap(long)]
    pub name: String,
    pub addr: NodeId,
}

#[derive(Debug, Clone, Parser)]
pub struct RemoveNode {
    #[clap(long)]
    pub name: String,
}

#[derive(Debug, Clone, Parser)]
pub struct ListNodes {}

#[derive(Debug, Clone, Parser)]
pub struct SystemInfo {
    pub id: Vec<String>,
}

#[derive(Debug, Clone, Parser)]
pub struct ListTasks {
    pub id: Vec<String>,
}

#[derive(Debug, Clone, Parser)]
pub struct KillTask {
    pub id: String,
    pub pid: u32,
}

#[derive(Debug, Clone, Parser)]
pub struct Shutdown {
    pub id: Vec<String>,
}

#[derive(Debug, Clone, Parser)]
pub struct PlayAudio {
    pub id: Vec<String>,
    #[clap(long)]
    pub source: AudioSource,
}
