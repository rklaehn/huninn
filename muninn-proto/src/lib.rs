use std::{fmt, time::Duration};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    ListProcesses,
    KillProcess(u32),
    PlayAudio(AudioSource),
    GetSystemInfo,
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AudioSource {
    WakeUp,
    GoToBed,
    RickRoll,
    Url(String),
}

impl fmt::Display for AudioSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioSource::WakeUp => write!(f, "WakeUp"),
            AudioSource::GoToBed => write!(f, "GoToBed"),
            AudioSource::RickRoll => write!(f, "RickRoll"),
            AudioSource::Url(url) => write!(f, "Url({})", url),
        }
    }
}

impl std::str::FromStr for AudioSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "wakeup" => Ok(AudioSource::WakeUp),
            "gotobed" => Ok(AudioSource::GoToBed),
            "rickroll" => Ok(AudioSource::RickRoll),
            _ if s.starts_with("url(") && s.ends_with(")") => {
                let url = &s[4..s.len() - 1];  // Extract the URL inside the "Url()" format
                Ok(AudioSource::Url(url.to_string()))
            }
            _ => Err(format!("Invalid string: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListProcessesResponse {
    pub tasks: Vec<(u32, String)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SysInfoResponse {
    pub hostname: String,
    pub uptime: Duration,
}

pub const ALPN: &[u8] = b"muninn";
pub const MAX_REQUEST_SIZE: usize = 1024;
pub const MAX_RESPONSE_SIZE: usize = 1024 * 1024;
