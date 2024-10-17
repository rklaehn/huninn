use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    ListProcesses,
    KillProcess(u32),
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListProcessesResponse {
    pub tasks: Vec<(u32, String)>,
}

pub const ALPN: &[u8] = b"muninn";
pub const MAX_REQUEST_SIZE: usize = 1024;
pub const MAX_RESPONSE_SIZE: usize = 1024 * 1024;
