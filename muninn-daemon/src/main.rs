use bytes::Bytes;
use clap::Parser;
use futures::stream::StreamExt;
use iroh_net::{endpoint, ticket::NodeTicket, NodeId};
use std::collections::BTreeSet;

mod config;
use config::Config;

mod args;
use args::Args;

mod os;
use os::{
    get_uptime, kill_process_by_id, list_processes, play_audio_on_all_devices,
    play_sound_on_default_device,
};

use muninn_proto::{AudioSource, ListProcessesResponse, Request};

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
            args::Subcommand::TestAudio(test_audio) => {
                let audio_data: Bytes = match test_audio.source {
                    AudioSource::WakeUp => WAKE_UP.into(),
                    AudioSource::GoToBed => GO_TO_BED.into(),
                    AudioSource::RickRoll => RICKROLL.into(),
                    AudioSource::Url(_) => {
                        anyhow::bail!("URL playback not implemented");
                    }
                };
                play_sound_on_default_device(audio_data)?;
            }
        }
        return Ok(());
    }
    println!("I am {}", config.secret_key.public());
    let endpoint = iroh_net::Endpoint::builder()
        .discovery(Box::new(
            iroh_net::discovery::pkarr::PkarrPublisher::n0_dns(config.secret_key.clone()),
        ))
        .secret_key(config.secret_key.clone())
        .alpns(vec![muninn_proto::ALPN.into()])
        .bind()
        .await?;
    endpoint.watch_home_relay().next().await;
    let info = endpoint.node_addr().await?;
    tracing::info!("Listening on {:?}", info);
    let ticket = NodeTicket::from(info);
    println!("My ticket: {}", ticket);
    while let Some(incoming) = endpoint.accept().await {
        tokio::spawn(handle_incoming(incoming, config.allowed_nodes.clone()));
    }
    Ok(())
}

const WAKE_UP: &[u8] = include_bytes!("../assets/wake_up.mp3");
const GO_TO_BED: &[u8] = include_bytes!("../assets/go_to_bed.mp3");
const RICKROLL: &[u8] = include_bytes!("../assets/rickroll.mp3");

async fn handle_incoming(
    incoming: endpoint::Incoming,
    allowed_nodes: BTreeSet<NodeId>,
) -> anyhow::Result<()> {
    let accepting = incoming.accept()?;
    let connection = accepting.await?;
    let remote_node_id = iroh_net::endpoint::get_remote_node_id(&connection)?;
    if !allowed_nodes.contains(&remote_node_id) {
        connection.close(1u32.into(), b"unauthorized node");
        tracing::info!(
            "Unauthorized node attempted to connect: {:?}",
            remote_node_id
        );
        return Ok(());
    }
    let (mut send, mut recv) = connection.accept_bi().await?;
    let msg = recv.read_to_end(muninn_proto::MAX_REQUEST_SIZE).await?;
    let msg = postcard::from_bytes::<muninn_proto::Request>(&msg)?;
    match msg {
        Request::ListProcesses => {
            tracing::info!("Listing processes");
            let tasks = list_processes();
            let response = ListProcessesResponse { tasks };
            let response = postcard::to_allocvec(&response)?;
            send.write_all(&response).await?;
            send.finish()?;
            connection.closed().await;
        }
        Request::KillProcess(pid) => {
            tracing::info!("Killing process {}", pid);
            let res = kill_process_by_id(pid);
            let response = res
                .err()
                .map(|e| e.to_string())
                .unwrap_or_else(|| "OK".to_string());
            let response = postcard::to_allocvec(&response)?;
            send.write_all(&response).await?;
            send.finish()?;
            connection.closed().await;
        }
        Request::GetSystemInfo => {
            tracing::info!("Getting system info");
            let uptime = get_uptime()?;
            let hostname = hostname::get()?
                .into_string()
                .map_err(|_| anyhow::anyhow!("Invalid hostname"))?;
            let response = muninn_proto::SysInfoResponse { uptime, hostname };
            let response = postcard::to_allocvec(&response)?;
            send.write_all(&response).await?;
            send.finish()?;
            connection.closed().await;
        }
        Request::PlayAudio(source) => {
            let audio_data: Bytes = match source {
                AudioSource::WakeUp => WAKE_UP.into(),
                AudioSource::GoToBed => GO_TO_BED.into(),
                AudioSource::RickRoll => RICKROLL.into(),
                AudioSource::Url(url) => {
                    anyhow::bail!("URL playback not implemented: {}", url);
                }
            };
            let response = play_sound_on_default_device(audio_data);
            let response = match response {
                Ok(_) => "OK".to_string(),
                Err(e) => e.to_string(),
            };
            let response = postcard::to_allocvec(&response)?;
            send.write_all(&response).await?;
            send.finish()?;
            connection.closed().await;
        }
        Request::Shutdown => {
            // shutdown_system();
        }
    }
    connection.closed().await;
    Ok(())
}
