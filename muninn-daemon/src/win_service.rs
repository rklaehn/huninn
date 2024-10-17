#[macro_use]
extern crate windows_service;

use std::ffi::OsString;
use windows_service::service_dispatcher;

define_windows_service!(ffi_service_main, my_service_main);

fn my_service_main(arguments: Vec<OsString>) {
    // The entry point where execution will start on a background thread after a call to
    // `service_dispatcher::start` from `main`.
}

fn main() -> Result<(), windows_service::Error> {
    // Register generated `ffi_service_main` with the system and start the service, blocking
    // this thread until the service is stopped.
    service_dispatcher::start("myservice", ffi_service_main)?;
    Ok(())
}
