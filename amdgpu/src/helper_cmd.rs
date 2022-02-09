//! AMD GUI helper communication toolkit

use std::io::{Read, Write};
use std::ops::Deref;
use std::os::unix::net::UnixStream;

#[derive(Debug, thiserror::Error)]
pub enum GuiHelperError {
    #[error("GUI Helper socket file not found. Is service running?")]
    NoSockFile,
    #[error("Failed to connect to /var/lib/amdfand/helper.sock. {0}")]
    UnableToConnect(#[from] std::io::Error),
    #[error("Failed to service helper command. {0}")]
    Serialize(#[from] ron::Error),
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
    ReloadConfig { pid: Pid },
    FanServices,
    SaveFanConfig { path: String, content: String },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Response {
    NoOp,
    Services(Vec<Pid>),
    ConfigFileSaved,
    ConfigFileSaveFailed(String),
}

pub fn sock_file() -> std::path::PathBuf {
    std::path::Path::new("/tmp").join("amdgui-helper.sock")
}

pub fn send_command(cmd: Command) -> crate::Result<Response> {
    let sock_path = sock_file();

    if !sock_path.exists() {
        return Err(GuiHelperError::NoSockFile.into());
    }

    let mut stream = UnixStream::connect(&sock_path).map_err(GuiHelperError::UnableToConnect)?;
    let s = ron::to_string(&cmd).map_err(GuiHelperError::Serialize)?;
    if stream.write_all(format!("{}\n", s).as_bytes()).is_ok() {
        log::info!("Command send");
    }

    let res: Response = {
        let mut s = String::with_capacity(100);
        let _ = stream.read_to_string(&mut s);
        ron::from_str(&s).map_err(GuiHelperError::Serialize)?
    };

    Ok(res)
}
