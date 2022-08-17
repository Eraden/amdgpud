use std::fmt::Debug;
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::net::Shutdown;
use std::ops::Deref;
use std::os::unix::net::UnixStream;

use serde::de::DeserializeOwned;
use serde::Serialize;

pub mod helper_cmd;
pub mod ports;

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Pid(pub i32);

impl Deref for Pid {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn handle_connection<HandleCmd, Cmd, Res>(stream: UnixStream, handle_command: HandleCmd)
where
    HandleCmd: FnOnce(Service<Res>, Cmd) + Copy,
    Cmd: DeserializeOwned + Serialize + Debug,
    Res: DeserializeOwned + Serialize + Debug + PidResponse,
{
    let mut service = Service::<Res>::new(stream);

    let command = match service.read_command() {
        Some(s) => s,
        _ => return service.kill(),
    };

    log::info!("Incoming {:?}", command);
    let cmd = match ron::from_str::<Cmd>(command.trim()) {
        Ok(cmd) => cmd,
        Err(e) => {
            log::warn!("Invalid message {:?}. {:?}", command, e);
            return service.kill();
        }
    };
    handle_command(service, cmd);
}

pub trait PidResponse: Sized {
    fn kill_response() -> Self;
}

pub struct Service<Response>(UnixStream, PhantomData<Response>)
where
    Response: serde::Serialize + Debug + PidResponse;

impl<Response> Service<Response>
where
    Response: serde::Serialize + Debug + PidResponse,
{
    pub fn new(file: UnixStream) -> Self {
        Self(file, Default::default())
    }

    /// Serialize and send command
    pub fn write_response(&mut self, res: Response) {
        write_response(&mut self.0, res)
    }

    /// Read from `.sock` file new line separated commands
    pub fn read_command(&mut self) -> Option<String> {
        read_command(&mut self.0)
    }

    /// Close connection with no operation response
    pub fn kill(mut self) {
        self.write_response(Response::kill_response());
        self.close();
    }

    pub fn close(self) {
        let _ = self.0.shutdown(Shutdown::Both);
    }
}

/// Serialize and send command
pub fn write_response<Response>(file: &mut UnixStream, res: Response)
where
    Response: serde::Serialize + Debug,
{
    match ron::to_string(&res) {
        Ok(buffer) => match file.write_all(buffer.as_bytes()) {
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
pub fn read_command(file: &mut UnixStream) -> Option<String> {
    let mut command = String::with_capacity(100);
    log::info!("Reading stream...");
    read_line(file, &mut command);
    if command.is_empty() {
        return None;
    }
    Some(command)
}

pub fn read_line(stream: &mut UnixStream, command: &mut String) {
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
