use clap::Parser;
use iroh_net::NodeId;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(Debug, Clone, Parser)]
pub enum Subcommand {
    AddNode(AddNode),
    ListTasks(ListTasks),
    Shutdown(Shutdown),
}

#[derive(Debug, Clone, Parser)]
pub struct AddNode {
    #[clap(long)]
    pub name: String,
    pub addr: NodeId,
}

#[derive(Debug, Clone, Parser)]
pub struct ListTasks {
    pub id: Vec<String>,
}

#[derive(Debug, Clone, Parser)]
pub struct Shutdown {
    pub id: Vec<String>,
}
