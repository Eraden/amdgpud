use amdgpu::utils;
use amdgpu_config::{fan, monitor};

#[derive(Debug, thiserror::Error)]
pub enum AmdMonError {
    #[error("Mon AMD GPU card was found")]
    NoHwMon,
    #[error("Failed to access {path:?}. {io}")]
    Io { io: std::io::Error, path: String },
    #[error("{0}")]
    MonConfigError(#[from] monitor::ConfigError),
    #[error("{0}")]
    FanConfigError(#[from] fan::ConfigError),
    #[error("{0}")]
    AmdUtils(#[from] utils::AmdGpuError),
    #[error("{0}")]
    Csv(#[from] csv::Error),
    #[error("AMD GPU temperature is malformed. It should be number. {0:?}")]
    NonIntTemp(std::num::ParseIntError),
    #[error("AMD GPU fan speed is malformed. It should be number. {0:?}")]
    NonIntPwm(std::num::ParseIntError),
    #[error("Monitor format is not valid. Available values are: short, s, long l, verbose and v")]
    InvalidMonitorFormat,
    #[error("Failed to read AMD GPU temperatures from tempX_input. No input was found")]
    EmptyTempSet,
}
