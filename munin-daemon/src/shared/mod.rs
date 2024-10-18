use bytes::Bytes;
use futures::stream::StreamExt;
use iroh_net::{endpoint, ticket::NodeTicket, NodeId};
use std::collections::BTreeSet;

mod config;
pub use config::{munin_data_root, Config};

mod os;
use os::{get_uptime, kill_process_by_id, list_processes, play_sound_on_default_device};

use munin_proto::{AudioSource, ListProcessesResponse, Request};

pub async fn run_daemon(
    config: Config,
    mut shutdown: tokio::sync::mpsc::UnboundedReceiver<()>,
) -> anyhow::Result<()> {
    println!("I am {}", config.secret_key.public());
    let endpoint = iroh_net::Endpoint::builder()
        .discovery(Box::new(
            iroh_net::discovery::pkarr::PkarrPublisher::n0_dns(config.secret_key.clone()),
        ))
        .secret_key(config.secret_key.clone())
        .alpns(vec![munin_proto::ALPN.into()])
        .bind()
        .await?;
    endpoint.watch_home_relay().next().await;
    let info = endpoint.node_addr().await?;
    tracing::info!("Listening on {:?}", info);
    let ticket = NodeTicket::from(info);
    println!("My ticket: {}", ticket);
    loop {
        tokio::select! {
            _ = shutdown.recv() => {
                break;
            }
            Some(incoming) = endpoint.accept() => {
                tokio::spawn(handle_incoming(incoming, config.allowed_nodes.clone()));
            }
        }
    }
    Ok(())
}

const WAKE_UP: &[u8] = include_bytes!("../../assets/wake_up.mp3");
const ALARM: &[u8] = include_bytes!("../../assets/alarm.mp3");
const RICKROLL: &[u8] = include_bytes!("../../assets/rickroll.mp3");

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
    let msg = recv.read_to_end(munin_proto::MAX_REQUEST_SIZE).await?;
    let msg = postcard::from_bytes::<munin_proto::Request>(&msg)?;
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
            let response = munin_proto::SysInfoResponse { uptime, hostname };
            let response = postcard::to_allocvec(&response)?;
            send.write_all(&response).await?;
            send.finish()?;
            connection.closed().await;
        }
        Request::PlayAudio(source) => {
            let audio_data: Bytes = match source {
                AudioSource::WakeUp => WAKE_UP.into(),
                AudioSource::Alarm => ALARM.into(),
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
