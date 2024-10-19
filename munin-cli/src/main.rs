use std::str::FromStr;

use anyhow::Result;
use args::Subcommand;
use clap::Parser;
use config::Config;
use iroh_net::{ticket::NodeTicket, NodeAddr, NodeId};

mod args;
mod config;

/// Given a list of ids from the cli, return a list of node ids and their names
///
/// If the list is empty, return all nodes in the config
fn get_nodes(ids: Vec<String>, config: &config::Config) -> Result<Vec<(String, NodeAddr)>> {
    if ids.is_empty() {
        config
            .nodes
            .iter()
            .map(|(name, id)| Ok((name.clone(), (*id).into())))
            .collect()
    } else {
        ids.into_iter()
            .map(|id| {
                if let Ok(nodeid) = NodeId::from_str(&id) {
                    Ok((nodeid.to_string(), NodeAddr::from(nodeid)))
                } else if let Ok(ticket) = NodeTicket::from_str(&id) {
                    Ok((
                        ticket.node_addr().node_id.to_string(),
                        ticket.node_addr().clone(),
                    ))
                } else if let Some(nodeid) = config.nodes.get(&id) {
                    Ok((id, (*nodeid).into()))
                } else {
                    Err(anyhow::anyhow!("Neither node id nor valid alias: {}", id))
                }
            })
            .collect()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = config::Config::get_or_create()?;
    let pubkey = config.secret_key.public();
    let res = main_impl(config).await;
    if let Err(err) = &res {
        let text = format!("{:?}", err);
        if text.contains("unauthorized node") {
            println!("The remote node rejected the connection");
            println!("You need to add {pubkey} to the list of allowed nodes");
            std::process::exit(1);
        }
    }
    res
}

async fn main_impl(mut config: Config) -> anyhow::Result<()> {
    println!("I am {}", config.secret_key.public());
    let args = args::Args::parse();
    let create_endpoint = || {
        iroh_net::Endpoint::builder()
            .discovery(Box::new(iroh_net::discovery::dns::DnsDiscovery::n0_dns()))
            .secret_key(config.secret_key.clone())
            .bind()
    };
    match args.subcommand {
        Subcommand::ListTasks(list_tasks) => {
            let nodes = get_nodes(list_tasks.id, &config)?;
            let endpoint = create_endpoint().await?;
            for (name, id) in nodes {
                println!("Listing tasks for {}", name);
                let connection = endpoint.connect(id, munin_proto::ALPN).await?;
                let (mut send, mut recv) = connection.open_bi().await?;
                let request = munin_proto::Request::ListProcesses;
                let request = postcard::to_allocvec(&request)?;
                send.write_all(&request).await?;
                send.finish()?;
                let msg = recv.read_to_end(munin_proto::MAX_RESPONSE_SIZE).await?;
                let msg = postcard::from_bytes::<munin_proto::ListProcessesResponse>(&msg)?;
                for (pid, name) in msg.tasks {
                    println!("{}: {}", pid, name);
                }
                connection.close(0u32.into(), b"OK");
            }
        }
        Subcommand::KillTask(kill_task) => {
            let node = if let Ok(nodeid) = NodeId::from_str(&kill_task.id) {
                nodeid
            } else if let Some(nodeid) = config.nodes.get(&kill_task.id) {
                *nodeid
            } else {
                return Err(anyhow::anyhow!(
                    "Neither node id nor valid alias: {}",
                    kill_task.id
                ));
            };
            let endpoint = create_endpoint().await?;
            let connection = endpoint.connect(node.into(), munin_proto::ALPN).await?;
            let (mut send, mut recv) = connection.open_bi().await?;
            let request = munin_proto::Request::KillProcess(kill_task.pid);
            let request = postcard::to_allocvec(&request)?;
            send.write_all(&request).await?;
            send.finish()?;
            let msg = recv.read_to_end(munin_proto::MAX_RESPONSE_SIZE).await?;
            let msg = postcard::from_bytes::<String>(&msg)?;
            println!("{}", msg);
            connection.close(0u32.into(), b"OK");
        }
        Subcommand::SystemInfo(system_info) => {
            let nodes = get_nodes(system_info.id, &config)?;
            let endpoint = create_endpoint().await?;
            for (name, id) in nodes {
                println!("Getting system info for {}", name);
                let connection = endpoint.connect(id, munin_proto::ALPN).await?;
                let (mut send, mut recv) = connection.open_bi().await?;
                let request = munin_proto::Request::GetSystemInfo;
                let request = postcard::to_allocvec(&request)?;
                send.write_all(&request).await?;
                send.finish()?;
                let msg = recv.read_to_end(munin_proto::MAX_RESPONSE_SIZE).await?;
                let msg = postcard::from_bytes::<munin_proto::SysInfoResponse>(&msg)?;
                println!("Hostname: {}", msg.hostname);
                println!("Uptime: {:?}", msg.uptime);
                connection.close(0u32.into(), b"OK");
            }
        }
        Subcommand::PlayAudio(play_audio) => {
            let nodes = get_nodes(play_audio.id, &config)?;
            let endpoint = create_endpoint().await?;
            for (name, id) in nodes {
                println!("Playing audio on {}", name);
                let connection = endpoint.connect(id, munin_proto::ALPN).await?;
                let (mut send, mut recv) = connection.open_bi().await?;
                let request = munin_proto::Request::PlayAudio(play_audio.source.clone());
                let request = postcard::to_allocvec(&request)?;
                send.write_all(&request).await?;
                send.finish()?;
                let msg = recv.read_to_end(munin_proto::MAX_RESPONSE_SIZE).await?;
                let msg = postcard::from_bytes::<String>(&msg)?;
                println!("{}", msg);
                connection.close(0u32.into(), b"OK");
            }
        }
        Subcommand::Shutdown(shutdown) => {
            println!("Shutting down {:?}", shutdown.id);
        }
        Subcommand::AddNode(add_node) => {
            config.nodes.insert(add_node.name, add_node.addr);
            config.save()?;
        }
        Subcommand::RemoveNode(remove_node) => {
            if config.nodes.remove(&remove_node.name).is_some() {
                config.save()?;
            } else {
                println!("No alias with name {}", remove_node.name);
            }
        }
        Subcommand::ListNodes(_) => {
            for (name, id) in &config.nodes {
                println!("{}: {}", name, id);
            }
        }
    }
    Ok(())
}
