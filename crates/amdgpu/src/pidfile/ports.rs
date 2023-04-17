//! AMD GUI helper communication toolkit

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::pidfile::PidResponse;

#[derive(Debug, thiserror::Error)]
pub enum PortsError {
    #[error("AMD GPU ports socket file not found. Is service running?")]
    NoSockFile,
    #[error("Failed to connect to /tmp/amdgpu-ports.sock. {0}")]
    UnableToConnect(#[from] std::io::Error),
    #[error("Failed to ports command. {0}")]
    Serialize(#[from] ron::Error),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum OutputType {
    Reserved,
    #[serde(rename = "v")]
    Vga,
    #[serde(rename = "m")]
    MiniDvi,
    #[serde(rename = "h")]
    Hdmi,
    #[serde(rename = "a")]
    Audio,
    #[serde(rename = "o")]
    OpticalAudio,
    #[serde(rename = "d")]
    Dvi,
    #[serde(rename = "t")]
    Thunderbolt,
    #[serde(rename = "D")]
    DisplayPort,
    #[serde(rename = "M")]
    MiniDisplayPort,
    #[serde(rename = "f")]
    FireWire400,
    #[serde(rename = "p")]
    Ps2,
    #[serde(rename = "s")]
    Sata,
    #[serde(rename = "e")]
    ESata,
    #[serde(rename = "E")]
    Ethernet,
    #[serde(rename = "F")]
    FireWire800,
    #[serde(rename = "1")]
    UsbTypeA,
    #[serde(rename = "2")]
    UsbTypeB,
    #[serde(rename = "3")]
    UsbTypeC,
    #[serde(rename = "4")]
    MicroUsb,
    #[serde(rename = "5")]
    MimiUsb,
}

impl OutputType {
    pub fn to_coords(&self) -> (u32, u32) {
        match self {
            OutputType::Reserved => (0, 0),
            //
            OutputType::Vga => (0, 0),
            OutputType::MiniDvi => (80, 0),
            OutputType::Hdmi => (160, 0),
            OutputType::Audio => (240, 0),
            OutputType::OpticalAudio => (320, 0),
            //
            OutputType::Dvi => (0, 80),
            OutputType::Thunderbolt => (80, 80),
            OutputType::DisplayPort => (160, 80),
            OutputType::MiniDisplayPort => (240, 80),
            OutputType::FireWire400 => (320, 80),
            //
            OutputType::Ps2 => (0, 160),
            OutputType::Sata => (80, 160),
            OutputType::ESata => (160, 160),
            OutputType::Ethernet => (240, 160),
            OutputType::FireWire800 => (320, 160),
            //
            OutputType::UsbTypeA => (0, 240),
            OutputType::UsbTypeB => (80, 240),
            OutputType::UsbTypeC => (160, 240),
            OutputType::MicroUsb => (240, 240),
            OutputType::MimiUsb => (320, 240),
        }
    }
    pub fn name(&self) -> &str {
        match self {
            OutputType::Reserved => "-----",
            //
            OutputType::Vga => "Vga",
            OutputType::MiniDvi => "MiniDvi",
            OutputType::Hdmi => "Hdmi",
            OutputType::Audio => "Audio",
            OutputType::OpticalAudio => "OptimalAudio",
            //
            OutputType::Dvi => "Dvi",
            OutputType::Thunderbolt => "Thunderbolt",
            OutputType::DisplayPort => "DisplayPort",
            OutputType::MiniDisplayPort => "MiniDisplayPort",
            OutputType::FireWire400 => "FireWire400",
            //
            OutputType::Ps2 => "Ps2",
            OutputType::Sata => "Sata",
            OutputType::ESata => "ESata",
            OutputType::Ethernet => "Ethernet",
            OutputType::FireWire800 => "FireWire800",
            //
            OutputType::UsbTypeA => "UsbTypeA",
            OutputType::UsbTypeB => "UsbTypeB",
            OutputType::UsbTypeC => "UsbTypeC",
            OutputType::MicroUsb => "MicroUsb",
            OutputType::MimiUsb => "MimiUsb",
        }
    }

    pub fn parse_str(s: &str) -> Option<Self> {
        Some(match s {
            "DP" => Self::DisplayPort,
            "eDP" => Self::MiniDisplayPort,
            "DVI" => Self::Dvi,
            "HDMI" => Self::Hdmi,
            _ => return None,
        })
    }

    pub fn all() -> [OutputType; 20] {
        [
            OutputType::Vga,
            OutputType::MiniDvi,
            OutputType::Hdmi,
            OutputType::Audio,
            OutputType::OpticalAudio,
            OutputType::Dvi,
            OutputType::Thunderbolt,
            OutputType::DisplayPort,
            OutputType::MiniDisplayPort,
            OutputType::FireWire400,
            OutputType::Ps2,
            OutputType::Sata,
            OutputType::ESata,
            OutputType::Ethernet,
            OutputType::FireWire800,
            OutputType::UsbTypeA,
            OutputType::UsbTypeB,
            OutputType::UsbTypeC,
            OutputType::MicroUsb,
            OutputType::MimiUsb,
        ]
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Output {
    #[serde(rename = "c")]
    pub card: String,
    #[serde(rename = "t")]
    pub port_type: String,
    #[serde(rename = "T")]
    pub ty: Option<OutputType>,
    #[serde(rename = "m")]
    pub port_name: Option<String>,
    #[serde(rename = "n")]
    pub port_number: u8,
    #[serde(rename = "s")]
    pub status: Status,
    #[serde(rename = "M")]
    pub modes: Vec<OutputMode>,
    pub display_power_managment: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OutputMode {
    #[serde(rename = "w")]
    pub width: u16,
    #[serde(rename = "h")]
    pub height: u16,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Command {
    Ports,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Response {
    Ports(Vec<Output>),
    NoOp,
}

impl PidResponse for Response {
    fn kill_response() -> Self {
        Self::NoOp
    }
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
        info!("Command send");
    }

    let res: Response = {
        let mut s = String::with_capacity(100);
        let _ = stream.read_to_string(&mut s);
        ron::from_str(&s).map_err(PortsError::Serialize)?
    };

    Ok(res)
}
