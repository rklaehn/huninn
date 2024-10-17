use std::collections::BTreeSet;

use iroh_net::{endpoint, NodeId};
use sysinfo::System;

mod config;
use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = Config::get_or_create()?;
    println!("I am {}", config.secret_key.public());
    let endpoint = iroh_net::Endpoint::builder()
        .discovery(Box::new(
            iroh_net::discovery::pkarr::PkarrPublisher::n0_dns(config.secret_key.clone()),
        ))
        .secret_key(config.secret_key.clone())
        .alpns(vec![muninn_proto::ALPN.into()])
        .bind()
        .await?;

    while let Some(incoming) = endpoint.accept().await {
        tokio::spawn(handle_incoming(incoming, config.allowed_nodes.clone()));
    }
    Ok(())
}

async fn handle_incoming(
    incoming: endpoint::Incoming,
    allowed_nodes: BTreeSet<NodeId>,
) -> anyhow::Result<()> {
    let mut accepting = incoming.accept()?;
    let alpn = accepting.alpn().await?;
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
        muninn_proto::Request::ListTasks => {
            let tasks = list_processes();
            let response = muninn_proto::ListTasksResponse { tasks };
            let response = postcard::to_allocvec(&response)?;
            send.write_all(&response).await?;
            send.finish()?;
        }
        muninn_proto::Request::Shutdown => {
            // shutdown_system();
        }
    }
    connection.closed().await;
    Ok(())
}

fn list_processes() -> Vec<(u64, String)> {
    // Create a System object to get information about the system.
    let mut system = System::new_all();

    // Refresh the list of processes.
    system.refresh_all();

    // Create a vector to store the PIDs and names of the processes.
    let mut processes = Vec::new();
    for (pid, process) in system.processes() {
        processes.push((pid.as_u32().into(), process.name().to_string_lossy().into()));
    }

    processes
}

fn shutdown_system() {
    #[cfg(target_os = "linux")]
    {
        use libc::{reboot, LINUX_REBOOT_CMD_POWER_OFF};
        use std::process::exit;

        unsafe {
            if reboot(LINUX_REBOOT_CMD_POWER_OFF) != 0 {
                eprintln!("Failed to shut down system");
                exit(1);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("shutdown")
            .arg("-h")
            .arg("now")
            .status()
            .expect("Failed to shut down system");
    }

    #[cfg(target_os = "windows")]
    {
        use std::ptr;
        use winapi::um::winuser::ExitWindowsEx;
        use winapi::um::winuser::EWX_POWEROFF;

        unsafe {
            if ExitWindowsEx(EWX_POWEROFF, 0) == 0 {
                eprintln!("Failed to shut down system");
            }
        }
    }
}
