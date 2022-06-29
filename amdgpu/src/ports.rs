//! AMD GUI helper communication toolkit

use std::io::{Read, Write};
use std::ops::Deref;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum PortsError {
    #[error("AMD GPU ports socket file not found. Is service running?")]
    NoSockFile,
    #[error("Failed to connect to /tmp/amdgpu-ports.sock. {0}")]
    UnableToConnect(#[from] std::io::Error),
    #[error("Failed to ports command. {0}")]
    Serialize(#[from] ron::Error),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Output {
    #[serde(rename = "c")]
    pub card: String,
    #[serde(rename = "t")]
    pub port_type: String,
    #[serde(rename = "m")]
    pub port_name: Option<String>,
    #[serde(rename = "n")]
    pub port_number: u8,
    #[serde(rename = "s")]
    pub status: Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Status {
    #[serde(rename = "c")]
    Connected,
    #[serde(rename = "d")]
    Disconnected,
}

impl Default for Status {
    fn default() -> Self {
        Self::Disconnected
    }
}

impl Output {
    fn to_path(&self) -> PathBuf {
        PathBuf::new().join("/sys/class/drm").join(format!(
            "card{}-{}{}-{}",
            self.card,
            self.port_type,
            self.port_name
                .as_deref()
                .map(|s| format!("-{s}"))
                .unwrap_or_default(),
            self.port_number
        ))
    }

    fn status_path(&self) -> PathBuf {
        self.to_path().join("status")
    }

    pub fn read_status(&self) -> Option<Status> {
        Some(
            match std::fs::read_to_string(self.status_path()).ok()?.trim() {
                "connected" => Status::Connected,
                "disconnected" => Status::Disconnected,
                _ => return None,
            },
        )
    }
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Pid(pub i32);

impl Deref for Pid {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Command {
    Ports,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Response {
    Ports(Vec<Output>),
    NoOp,
}

pub fn sock_file() -> PathBuf {
    std::path::Path::new("/tmp").join("amdgpu-ports.sock")
}

pub fn send_command(cmd: Command) -> crate::Result<Response> {
    let sock_path = sock_file();

    if !sock_path.exists() {
        return Err(PortsError::NoSockFile.into());
    }

    let mut stream = UnixStream::connect(&sock_path).map_err(PortsError::UnableToConnect)?;
    let s = ron::to_string(&cmd).map_err(PortsError::Serialize)?;
    if stream.write_all(format!("{}\n", s).as_bytes()).is_ok() {
        log::info!("Command send");
    }

    let res: Response = {
        let mut s = String::with_capacity(100);
        let _ = stream.read_to_string(&mut s);
        ron::from_str(&s).map_err(PortsError::Serialize)?
    };

    Ok(res)
}
