use std::collections::BTreeSet;

use iroh_net::{endpoint, NodeId};
use sysinfo::System;

mod config;
use config::Config;

use muninn_proto::{self as proto, ListProcessesResponse, Request};

use std::io;

#[cfg(unix)]
use libc::{kill, SIGKILL};

#[cfg(windows)]
use winapi::um::handleapi::CloseHandle;
#[cfg(windows)]
use winapi::um::processthreadsapi::OpenProcess;
#[cfg(windows)]
use winapi::um::processthreadsapi::TerminateProcess;
#[cfg(windows)]
use winapi::um::winnt::PROCESS_TERMINATE;

#[cfg(windows)]
use winapi::shared::minwindef::DWORD;

#[cfg(target_os = "windows")]
mod win_service;

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
            let response = res.err().map(|e| e.to_string()).unwrap_or_else(|| "OK".to_string());
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

fn list_processes() -> Vec<(u32, String)> {
    // Create a System object to get information about the system.
    let mut system = System::new_all();

    // Refresh the list of processes.
    system.refresh_all();

    // Create a vector to store the PIDs and names of the processes.
    let mut processes = Vec::new();
    for (pid, process) in system.processes() {
        processes.push((pid.as_u32(), process.name().to_string_lossy().into()));
    }

    processes
}

#[cfg(unix)]
pub fn kill_process_by_id(pid: u32) -> io::Result<()> {
    let res = unsafe { kill(pid as i32, SIGKILL) };
    if res == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(windows)]
pub fn kill_process_by_id(pid: u32) -> io::Result<()> {
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid as DWORD);
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }

        let result = TerminateProcess(handle, 1); // Exit code 1
        CloseHandle(handle);

        if result != 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

pub fn shutdown_system() {
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
