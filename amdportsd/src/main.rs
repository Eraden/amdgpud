use amdgpu::ports::{sock_file, Command, Response};
use amdgpu::{ports::*, IoFailure};
use std::fs::{DirEntry, Permissions};
use std::io::{Read, Write};
use std::net::Shutdown;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
    let mut output = Output {
        card: it.next()?.strip_prefix("card")?.to_string(),
        port_type: it.next()?,
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

pub struct Service(UnixStream);

impl Service {
    /// Serialize and send command
    pub fn write_response(&mut self, res: Response) {
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
        log::error!("Failed to change gui helper socket file mode. {:?}", e);
    }

    while let Ok((stream, _addr)) = listener.accept() {
        handle_connection(stream, state.clone());
    }
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

fn handle_connection(stream: UnixStream, state: Arc<Mutex<Vec<Output>>>) {
    let mut service = Service(stream);

    let command = match service.read_command() {
        Some(s) => s,
        _ => return service.kill(),
    };

    log::info!("Incoming {:?}", command);
    let cmd = match ron::from_str::<Command>(command.trim()) {
        Ok(cmd) => cmd,
        Err(e) => {
            log::warn!("Invalid message {:?}. {:?}", command, e);
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
