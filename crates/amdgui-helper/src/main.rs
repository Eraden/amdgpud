//! Special daemon with root privileges. Since GUI should not have (and
//! sometimes can't have) root privileges and service processes are designed to
//! be as small as possible this is proxy.
//!
//! It is responsible for:
//! * Loading all amdfand processes. In order to do this process needs to be
//!   killed with signal 0 to check if it still is alive
//! * Reload amdfand process with signal SIGHUP
//! * Save changed config file
//!
//! It is using `/tmp/amdgui-helper.sock` file and `ron` serialization for
//! communication. After each operation connection is terminated so each command
//! needs new connection.
#![allow(clippy::non_octal_unix_permissions)]
extern crate eyra;

use std::ffi::OsStr;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;

use amdgpu::pidfile::helper_cmd::{Command, Response};
use amdgpu::pidfile::{handle_connection, Pid};
use amdgpu::IoFailure;
use tracing::{error, info, warn};

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
    tracing_subscriber::fmt::init();

    let mut lock = amdgpu::lock_file::PidLock::new("amdgui", String::from("helper"))?;
    lock.acquire()?;

    let sock_path = amdgpu::pidfile::helper_cmd::sock_file();
    let listener = {
        let _ = std::fs::remove_file(&sock_path);

        UnixListener::bind(&sock_path).map_err(|io| IoFailure {
            io,
            path: sock_path.clone(),
        })?
    };
    if let Err(e) = std::fs::set_permissions(&sock_path, Permissions::from_mode(0o777)) {
        error!("Failed to change gui helper socket file mode. {:?}", e);
    }

    while let Ok((stream, _addr)) = listener.accept() {
        handle_connection::<_, Command, Response>(stream, handle_command);
    }

    lock.release()?;
    Ok(())
}

pub type Service = amdgpu::pidfile::Service<Response>;

fn handle_command(service: Service, cmd: Command) {
    match cmd {
        Command::ReloadConfig { pid } => {
            info!("Reloading config file for pid {:?}", pid);
            handle_reload_config(service, pid);
        }
        Command::FanServices => handle_fan_services(service),
        Command::SaveFanConfig { path, content } => handle_save_fan_config(service, path, content),
    }
}

fn handle_save_fan_config(mut service: Service, path: String, content: String) {
    match std::fs::write(path, content) {
        Err(e) => service.write_response(Response::ConfigFileSaveFailed(format!("{:?}", e))),
        Ok(..) => service.write_response(Response::ConfigFileSaved),
    }
}

fn handle_fan_services(mut service: Service) {
    info!("Loading fan services");
    let services = read_fan_services();
    info!("Loaded fan services pid {:?}", services);
    service.write_response(Response::Services(services));
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
                        info!("Extension is {:?}", e.path().extension());
                        e.path().extension().and_then(OsStr::to_str) == Some("pid")
                    })
                    .ok()
                    .unwrap_or_default()
            })
            .filter_map(|e| {
                info!("Found entry {:?}", e);
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
        warn!("Directory /var/lib/amdfand not found");
        vec![]
    }
}
