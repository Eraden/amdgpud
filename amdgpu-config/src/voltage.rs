use amdgpu::utils::ensure_config;
use amdgpu::LogLevel;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    log_level: LogLevel,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log_level: LogLevel::Error,
        }
    }
}

impl Config {
    pub fn log_level(&self) -> LogLevel {
        self.log_level
    }
}

pub fn load_config(config_path: &str) -> Result<Config, ConfigError> {
    ensure_config::<Config, ConfigError, _>(config_path)
}
