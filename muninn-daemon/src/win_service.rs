#[cfg(windows)]
use std::ffi::OsString;
use std::time::Duration;
use std::os::windows::ffi::OsStringExt;
use windows_service::service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
use windows_service::service_dispatcher;

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
        status_handle.set_service_status(next_status.clone()).unwrap();

        // Simulate a running service (could be your logic here)
        std::thread::sleep(Duration::from_secs(60)); // Simulates service work

        // When the service is stopped, update the status
        let stopped_status = ServiceStatus {
            current_state: ServiceState::Stopped,
            ..next_status
        };
        status_handle.set_service_status(stopped_status).unwrap();
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