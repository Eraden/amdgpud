use serde::{Deserialize, Serialize};

use amdgpu::utils::ensure_config;
use amdgpu::LogLevel;

pub static DEFAULT_MONITOR_CONFIG_PATH: &str = "/etc/amdfand/monitor.toml";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    /// Minimal log level
    log_level: LogLevel,
    /// Time in milliseconds
    interval: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log_level: LogLevel::Error,
            interval: 5000,
        }
    }
}

impl Config {
    pub fn log_level(&self) -> LogLevel {
        self.log_level
    }

    pub fn interval(&self) -> u32 {
        self.interval
    }
}

pub fn load_config(config_path: &str) -> Result<Config, ConfigError> {
    let mut config: Config = ensure_config::<Config, ConfigError, _>(config_path)?;

    if config.interval < 100 {
        log::warn!(
            "Minimal interval is 100ms, overriding {}ms",
            config.interval
        );
        config.interval = 100;
    }

    Ok(config)
}
