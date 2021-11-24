mod card;
mod error;
pub mod hw_mon;
mod temp_input;
pub mod utils;

pub use card::*;
pub use error::*;
use serde::{Deserialize, Serialize};
pub use temp_input::*;

pub static CONFIG_DIR: &str = "/etc/amdfand";

pub static ROOT_DIR: &str = "/sys/class/drm";
pub static HW_MON_DIR: &str = "hwmon";

pub type Result<T> = std::result::Result<T, AmdGpuError>;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum LogLevel {
    /// A level lower than all log levels.
    Off,
    /// Corresponds to the `Error` log level.
    Error,
    /// Corresponds to the `Warn` log level.
    Warn,
    /// Corresponds to the `Info` log level.
    Info,
    /// Corresponds to the `Debug` log level.
    Debug,
    /// Corresponds to the `Trace` log level.
    Trace,
}

impl LogLevel {
    pub fn as_str(&self) -> &str {
        match self {
            LogLevel::Off => "OFF",
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }
}
