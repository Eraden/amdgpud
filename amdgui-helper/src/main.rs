//! Special daemon with root privileges. Since GUI should not have (and sometimes can't have) root
//! privileges and service processes are designed to be as small as possible this is proxy.
//!
//! It is responsible for:
//! * Loading all amdfand processes. In order to do this process needs to be killed with signal 0 to check if it still is alive
//! * Reload amdfand process with signal SIGHUP
//! * Save changed config file
//!
//! It is using `/tmp/amdgui-helper.sock` file and `ron` serialization for communication.
//! After each operation connection is terminated so each command needs new connection.
#![allow(clippy::non_octal_unix_permissions)]

use amdgpu::helper_cmd::{Command, Pid, Response};
use amdgpu::IoFailure;
use std::ffi::OsStr;
use std::fs::Permissions;
use std::io::{Read, Write};
use std::net::Shutdown;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] amdgpu::IoFailure),
    #[error("{0}")]
    Lock(#[from] amdgpu::AmdGpuError),
}

pub type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "DEBUG");
    }
    pretty_env_logger::init();

    let mut lock = amdgpu::lock_file::PidLock::new("amdgui", String::from("helper"))?;
    lock.acquire()?;

    let sock_path = amdgpu::helper_cmd::sock_file();
    let listener = {
        let _ = std::fs::remove_file(&sock_path);

        UnixListener::bind(&sock_path).map_err(|io| IoFailure {
            io,
            path: sock_path.clone(),
        })?
    };
    if let Err(e) = std::fs::set_permissions(&sock_path, Permissions::from_mode(0x777)) {
        log::error!("Failed to change gui helper socket file mode. {:?}", e);
    }

    while let Ok((stream, _addr)) = listener.accept() {
        handle_connection(stream);
    }

    lock.release()?;
    Ok(())
}

pub struct Service(UnixStream);

impl Service {
    /// Serialize and send command
    pub fn write_response(&mut self, res: amdgpu::helper_cmd::Response) {
        match ron::to_string(&res) {
            Ok(buffer) => match self.0.write_all(buffer.as_bytes()) {
                Ok(_) => {
                    log::info!("Response successfully written")
                }
                Err(e) => log::warn!("Failed to write response. {:?}", e),
            },
            Err(e) => {
                log::warn!("Failed to serialize response {:?}. {:?}", res, e)
            }
        }
    }

    /// Read from `.sock` file new line separated commands
    pub fn read_command(&mut self) -> Option<String> {
        let mut command = String::with_capacity(100);
        log::info!("Reading stream...");
        read_line(&mut self.0, &mut command);
        if command.is_empty() {
            return None;
        }
        Some(command)
    }

    /// Close connection with no operation response
    pub fn kill(mut self) {
        self.write_response(Response::NoOp);
        self.close();
    }

    pub fn close(self) {
        let _ = self.0.shutdown(Shutdown::Both);
    }
}

fn handle_connection(stream: UnixStream) {
    let mut service = Service(stream);

    let command = match service.read_command() {
        Some(s) => s,
        _ => return service.kill(),
    };

    log::info!("Incoming {:?}", command);
    let cmd = match ron::from_str::<amdgpu::helper_cmd::Command>(command.trim()) {
        Ok(cmd) => cmd,
        Err(e) => {
            log::warn!("Invalid message {:?}. {:?}", command, e);
            return service.kill();
        }
    };
    handle_command(service, cmd);
}

fn handle_command(mut service: Service, cmd: Command) {
    match cmd {
        Command::ReloadConfig { pid } => {
            log::info!("Reloading config file for pid {:?}", pid);
            handle_reload_config(service, pid);
        }
        Command::FanServices => handle_fan_services(service),
        Command::SaveFanConfig { path, content } => {
            handle_save_fan_config(&mut service, path, content)
        }
    }
}

fn handle_save_fan_config(service: &mut Service, path: String, content: String) {
    match std::fs::write(path, content) {
        Err(e) => service.write_response(Response::ConfigFileSaveFailed(format!("{:?}", e))),
        Ok(..) => service.write_response(Response::ConfigFileSaved),
    }
}

fn handle_fan_services(mut service: Service) {
    log::info!("Loading fan services");
    let services = read_fan_services();
    log::info!("Loaded fan services pid {:?}", services);
    service.write_response(Response::Services(services));
}

fn read_line(stream: &mut UnixStream, command: &mut String) {
    let mut buffer = [0];
    while stream.read_exact(&mut buffer).is_ok() {
        if buffer[0] == b'\n' {
            break;
        }
        match std::str::from_utf8(&buffer) {
            Ok(s) => {
                command.push_str(s);
            }
            Err(e) => {
                log::error!("Failed to read from client. {:?}", e);
                let _ = stream.shutdown(Shutdown::Both);
                continue;
            }
        }
    }
}

fn handle_reload_config(service: Service, pid: Pid) {
    unsafe {
        nix::libc::kill(pid.0, nix::sys::signal::Signal::SIGHUP as i32);
    }
    service.kill();
}

fn read_fan_services() -> Vec<Pid> {
    if let Ok(entry) = std::fs::read_dir("/var/lib/amdfand") {
        entry
            .filter(|e| {
                e.as_ref()
                    .map(|e| {
                        log::info!("Extension is {:?}", e.path().extension());
                        e.path().extension().and_then(OsStr::to_str) == Some("pid")
                    })
                    .ok()
                    .unwrap_or_default()
            })
            .filter_map(|e| {
                log::info!("Found entry {:?}", e);
                match e {
                    Ok(entry) => std::fs::read_to_string(entry.path())
                        .ok()
                        .and_then(|s| s.parse::<i32>().ok())
                        .filter(|pid| unsafe { nix::libc::kill(*pid, 0) } == 0),
                    _ => None,
                }
            })
            .map(Pid)
            .collect()
    } else {
        log::warn!("Directory /var/lib/amdfand not found");
        vec![]
    }
}
