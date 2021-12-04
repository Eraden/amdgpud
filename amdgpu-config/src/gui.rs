use amdgpu::utils::ensure_config;
use amdgpu::LogLevel;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// Minimal log level
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
    let config: Config = ensure_config::<Config, ConfigError, _>(config_path)?;

    Ok(config)
}

#[cfg(test)]
mod serde_tests {
    use crate::gui::Config;

    #[test]
    fn serialize() {
        let res = toml::to_string(&Config::default());
        assert!(res.is_ok());
    }

    #[test]
    fn deserialize() {
        let res = toml::from_str::<Config>(&toml::to_string(&Config::default()).unwrap());
        assert!(res.is_ok());
    }
}
