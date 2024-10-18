use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;
use std::ptr;
use std::time::Duration;
use windows_service::service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
use windows_service::service_dispatcher;

mod shared;
use shared::{muninn_data_root, run_daemon, Config};

const SERVICE_NAME: &str = "Muninn-Service";

fn main() -> windows_service::Result<()> {
    // Start the service
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}

extern "system" fn ffi_service_main(num_args: u32, raw_args: *mut *mut u16) {
    let args = parse_service_arguments(num_args, raw_args);
    // Create a service control handler
    if let Ok(status_handle) = service_control_handler::register(SERVICE_NAME, service_handler) {
        let next_status = ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::from_secs(10),
            process_id: None,
        };
        status_handle
            .set_service_status(next_status.clone())
            .expect("set service status failed");

        // Simulate a running service (could be your logic here)
        let rt = tokio::runtime::Runtime::new().expect("create tokio runtime failed");
        let config = Config::get_or_create().expect("get or create config failed");
        let id = config.secret_key.public().to_string();
        let data_dir = muninn_data_root().unwrap();
        log_event(&format!(
            "Starting muninn-daemon with id {} and data dir: {}",
            id,
            data_dir.display()
        ));
        rt.block_on(run_daemon(config)).expect("run daemon failed");

        // When the service is stopped, update the status
        let stopped_status = ServiceStatus {
            current_state: ServiceState::Stopped,
            ..next_status
        };
        status_handle
            .set_service_status(stopped_status)
            .expect("set service status failed");
    }
}

// This function handles control events like STOP
fn service_handler(control_event: ServiceControl) -> ServiceControlHandlerResult {
    match control_event {
        ServiceControl::Stop => ServiceControlHandlerResult::NoError,
        _ => ServiceControlHandlerResult::NotImplemented,
    }
}

// Parse the raw service arguments into a Vec<OsString>
fn parse_service_arguments(num_args: u32, raw_args: *mut *mut u16) -> Vec<OsString> {
    let args_ptrs = unsafe { std::slice::from_raw_parts(raw_args, num_args as usize) };
    args_ptrs
        .iter()
        .map(|&arg_ptr| {
            if arg_ptr.is_null() {
                OsString::new()
            } else {
                unsafe {
                    // Find the null-terminated end of the string
                    let len = (0..).take_while(|&i| *arg_ptr.add(i) != 0).count();
                    OsString::from_wide(std::slice::from_raw_parts(arg_ptr, len))
                }
            }
        })
        .collect()
}

fn to_lpcwstr(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<u16>>()
}

fn log_event(message: &str) {
    use winapi::um::winbase::{DeregisterEventSource, RegisterEventSourceW, ReportEventW};
    use winapi::um::winnt::EVENTLOG_SUCCESS;
    use winapi::um::winnt::{HANDLE, LPCWSTR};

    unsafe {
        // Register an event source with the event log system
        let source_name = to_lpcwstr(SERVICE_NAME);
        let event_source: HANDLE = RegisterEventSourceW(ptr::null(), source_name.as_ptr());

        if event_source.is_null() {
            eprintln!("Failed to register event source");
            return;
        }

        // Convert the message to LPCWSTR
        let message_wide = to_lpcwstr(message);
        let message_ptrs: [LPCWSTR; 1] = [message_wide.as_ptr()];

        // Log the event
        ReportEventW(
            event_source,
            EVENTLOG_SUCCESS,                // Type of event (success, error, etc.)
            0,                               // Category
            0,                               // Event ID
            ptr::null_mut(),                 // User SID (none) - must be mutable
            1,                               // Number of strings
            0,                               // Size of binary data
            message_ptrs.as_ptr() as *mut _, // Cast to *mut *const u16
            ptr::null_mut(),                 // Binary data (none) - must be mutable
        );

        // Deregister the event source when done
        DeregisterEventSource(event_source);
    }
}
