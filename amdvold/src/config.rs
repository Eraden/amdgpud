use amdgpu::LogLevel;
use std::io::ErrorKind;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {}

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

pub fn load_config(config_path: &str) -> crate::Result<Config> {
    let config = match std::fs::read_to_string(config_path) {
        Ok(s) => toml::from_str(&s).unwrap(),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            let config = Config::default();
            std::fs::write(config_path, toml::to_string(&config).unwrap())?;
            config
        }
        Err(e) => {
            log::error!("{:?}", e);
            panic!();
        }
    };
    Ok(config)
}
