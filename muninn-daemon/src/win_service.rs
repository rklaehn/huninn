#[cfg(windows)]
#[macro_use]
extern crate windows_service;

#[cfg(windows)]
mod service {
    use std::ffi::OsString;
    use std::time::Duration;
    use windows_service::service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    };
    use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
    use windows_service::service_dispatcher;

    const SERVICE_NAME: &str = "my_rust_service";

    fn main() -> windows_service::Result<()> {
        // Start the service
        service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
        Ok(())
    }

    fn ffi_service_main(_arguments: Vec<OsString>) {
        // Create a service control handler
        if let Ok(status_handle) = service_control_handler::register(SERVICE_NAME, service_handler)
        {
            let next_status = ServiceStatus {
                service_type: ServiceType::OWN_PROCESS,
                current_state: ServiceState::Running,
                controls_accepted: ServiceControlAccept::STOP,
                win32_exit_code: ServiceExitCode::Win32(0),
                service_specific_exit_code: 0,
                check_point: 0,
                wait_hint: Duration::from_secs(10).as_secs() as u32,
                process_id: None,
            };
            status_handle.set_service_status(next_status).unwrap();

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
}
