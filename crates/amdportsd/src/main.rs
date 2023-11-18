use std::fs::{DirEntry, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use amdgpu::pidfile::ports::{sock_file, Command, Response, *};
use amdgpu::IoFailure;
use tracing_subscriber::EnvFilter;

extern crate eyra;

fn parse_output(entry: DirEntry) -> Option<Output> {
    let ty = entry.file_type().ok()?;
    if ty.is_dir() {
        return None;
    }
    let file_name = entry.file_name();
    let path = file_name.to_str()?;
    let mut it = path
        .split('-')
        .map(String::from)
        .collect::<Vec<_>>()
        .into_iter();

    let modes = std::fs::read_to_string(entry.path().join("modes"))
        .unwrap_or_default()
        .lines()
        .filter_map(|s| {
            let mut it = s.split('x');
            let width = it.next().and_then(|s| s.parse::<u16>().ok())?;
            let height = it.next().and_then(|s| s.parse::<u16>().ok())?;
            Some(OutputMode { width, height })
        })
        .collect::<Vec<_>>();

    let card = it.next()?.strip_prefix("card")?.to_string();
    let port_type = it.next()?;
    let dpms = std::fs::read_to_string(entry.path().join("dpms"))
        .unwrap_or_else(|e| {
            tracing::error!("{}", e);
            "Off".into()
        })
        .to_lowercase();
    tracing::info!("Display Power Management System is {:?}", dpms);

    let mut output = Output {
        card,
        ty: OutputType::parse_str(&port_type),
        port_type,
        modes,
        display_power_managment: dpms.trim() == "on",
        ..Default::default()
    };
    let mut it = it.rev();
    output.port_number = it.next()?.parse().ok()?;

    let mut it = it.rev().peekable();

    if it.peek().is_some() {
        output.port_name = Some(it.collect::<Vec<_>>().join("-"));
    }

    output.status = output.read_status()?;

    Some(output)
}

async fn read_outputs(state: Arc<Mutex<Vec<Output>>>) {
    loop {
        let outputs = std::fs::read_dir("/sys/class/drm")
            .unwrap()
            .filter_map(|r| r.ok())
            .filter(|e| {
                e.path()
                    .to_str()
                    .map(|s| s.contains("card"))
                    .unwrap_or_default()
            })
            .filter_map(parse_output)
            .collect::<Vec<_>>();
        if let Ok(mut lock) = state.lock() {
            *lock = outputs;
        }

        tokio::time::sleep(Duration::from_millis(1_000 / 3)).await;
    }
}

pub type Service = amdgpu::pidfile::Service<Response>;

async fn service(state: Arc<Mutex<Vec<Output>>>) {
    let sock_path = sock_file();
    let listener = {
        let _ = std::fs::remove_file(&sock_path);

        UnixListener::bind(&sock_path)
            .map_err(|io| IoFailure {
                io,
                path: sock_path.clone(),
            })
            .expect("Creating pid file for ports failed")
    };
    if let Err(e) = std::fs::set_permissions(&sock_path, Permissions::from_mode(0o777)) {
        tracing::error!("Failed to change gui helper socket file mode. {:?}", e);
    }

    while let Ok((stream, _addr)) = listener.accept() {
        handle_connection(stream, state.clone());
    }
}

fn handle_connection(stream: UnixStream, state: Arc<Mutex<Vec<Output>>>) {
    let mut service = Service::new(stream);

    let command = match service.read_command() {
        Some(s) => s,
        _ => return service.kill(),
    };

    tracing::info!("Incoming {:?}", command);
    let cmd = match ron::from_str::<Command>(command.trim()) {
        Ok(cmd) => cmd,
        Err(e) => {
            tracing::warn!("Invalid message {:?}. {:?}", command, e);
            return service.kill();
        }
    };
    handle_command(service, cmd, state);
}

fn handle_command(mut service: Service, cmd: Command, state: Arc<Mutex<Vec<Output>>>) {
    match cmd {
        Command::Ports => {
            if let Ok(outputs) = state.lock() {
                service.write_response(Response::Ports(outputs.iter().map(Clone::clone).collect()));
            }
        }
    }
}

fn main() {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let state = Arc::new(Mutex::new(Vec::new()));

    executor.block_on(async {
        let sync = read_outputs(state.clone());
        let handle = service(state);

        tokio::join!(sync, handle);
    });
}
