use std::{io, time::Duration};

use bytes::Bytes;
use cpal::traits::HostTrait;
use rodio::DeviceTrait;

pub fn list_processes() -> Vec<(u32, String)> {
    // Create a System object to get information about the system.
    let mut system = sysinfo::System::new_all();

    // Refresh the list of processes.
    system.refresh_all();

    // Create a vector to store the PIDs and names of the processes.
    let mut processes = Vec::new();
    for (pid, process) in system.processes() {
        processes.push((pid.as_u32(), process.name().to_string_lossy().into()));
    }

    processes
}

pub fn get_uptime() -> io::Result<Duration> {
    #[cfg(target_os = "linux")]
    {
        use libc::sysinfo;
        let mut info = std::mem::MaybeUninit::<libc::sysinfo>::uninit();
        let res = unsafe { sysinfo(info.as_mut_ptr()) };
        if res != 0 {
            return Err(io::Error::last_os_error());
        }
        let info = unsafe { info.assume_init() };
        let uptime = info.uptime as u64;
        return Ok(Duration::from_secs(uptime));
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        // Use sysctl to get the uptime in seconds
        let output = Command::new("sysctl")
            .arg("-n")
            .arg("kern.boottime")
            .output()?;
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to get uptime"));
        }
        let output_str = String::from_utf8_lossy(&output.stdout);
        let boot_time = parse_boottime(&output_str)?;

        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time before UNIX EPOCH");

        let uptime = current_time.as_secs() - boot_time;
        return Ok(Duration::from_secs(uptime));
    }

    #[cfg(windows)]
    {
        use winapi::um::sysinfoapi::GetTickCount64;
        let uptime_ms = unsafe { GetTickCount64() };
        return Ok(Duration::from_millis(uptime_ms));
    }

    // If for some reason we are on an unsupported platform, return an error.
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    {
        return Err(io::Error::new(io::ErrorKind::Other, "Unsupported platform"));
    }
}

// Helper function to parse the boot time on macOS from sysctl output
#[cfg(target_os = "macos")]
fn parse_boottime(output: &str) -> io::Result<u64> {
    // Example sysctl output: "{ sec = 1632438400, usec = 123456 } Wed Sep 24 12:34:56 2021"
    let start = output.find("sec = ").ok_or(io::Error::new(io::ErrorKind::Other, "Invalid sysctl output"))?;
    let end = output[start..].find(',').ok_or(io::Error::new(io::ErrorKind::Other, "Invalid sysctl output"))?;
    let boot_time_str = &output[start + 6..start + end]; // 6 is length of "sec = "
    let boot_time: u64 = boot_time_str.trim().parse().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to parse boot time"))?;
    Ok(boot_time)
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

pub fn kill_process_by_id(pid: u32) -> io::Result<()> {
    #[cfg(unix)]
    {
        use libc::{kill, SIGKILL};

        let res = unsafe { kill(pid as i32, SIGKILL) };
        if res == 0 {
            return Ok(());
        } else {
            return Err(io::Error::last_os_error());
        }
    }

    #[cfg(windows)]
    {
        use winapi::um::handleapi::CloseHandle;
        use winapi::um::processthreadsapi::{OpenProcess, TerminateProcess};
        use winapi::um::winnt::PROCESS_TERMINATE;
        use winapi::shared::minwindef::DWORD;

        unsafe {
            let handle = OpenProcess(PROCESS_TERMINATE, 0, pid as DWORD);
            if handle.is_null() {
                return Err(io::Error::last_os_error());
            }

            let result = TerminateProcess(handle, 1); // Exit code 1
            CloseHandle(handle);

            if result != 0 {
                return Ok(());
            } else {
                return Err(io::Error::last_os_error());
            }
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        return Err(io::Error::new(io::ErrorKind::Other, "Unsupported platform"));
    }
}

pub fn play_audio_on_all_devices(audio_data: Bytes) -> anyhow::Result<Vec<String>> {
    // Get the default host and all available output devices
    let host = cpal::default_host();
    let devices = host.output_devices().expect("Failed to get output devices").collect::<Vec<_>>();

    if devices.len() == 0 {
        eprintln!("No audio output devices found!");
        return Ok(vec![]);
    } else {
        for device in &devices {
            println!("Found output device: {}", device.name().unwrap_or_else(|_| "Unknown Device".to_string()));
        }
    }

    let mut results = Vec::new();
    // Iterate over all output devices and play the sound
    std::thread::scope(|scope| {
        for device in devices {
            let device_name = device.name().unwrap_or_else(|_| "Unknown Device".to_string());
            println!("Playing on: {}", device_name);

            // Create a new output stream and stream handle for each device
            if let Ok((_stream, stream_handle)) = rodio::OutputStream::try_from_device(&device) {
                let audio_data = audio_data.clone();
                results.push(format!("Playing sound on device: {}", device_name));
                scope.spawn(move || {
                    // Create a cursor for the embedded audio data
                    let cursor = io::Cursor::new(audio_data);
                    play_sound_on_device(cursor, &stream_handle);
                });
            } else {
                results.push(format!("Error: failed to create stream for device: {}", device_name));
            }
        }
    });

    Ok(results)
}

// Helper function to play the sound on a single device
fn play_sound_on_device(audio_data: io::Cursor<Bytes>, stream_handle: &rodio::OutputStreamHandle) {
    let sink = rodio::Sink::try_new(stream_handle).unwrap();
    let source = rodio::Decoder::new(audio_data).unwrap();
    sink.append(source);
    sink.sleep_until_end();  // Block until the sound finishes playing
}