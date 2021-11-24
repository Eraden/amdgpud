use crate::command::FanError;
use crate::config::ConfigError;
use amdgpu::AmdGpuError;

#[derive(Debug, thiserror::Error)]
pub enum AmdFanError {
    #[error("Vendor is not AMD")]
    NotAmdCard,
    #[error("Monitor format is not valid. Available values are: short, s, long l, verbose and v")]
    InvalidMonitorFormat,
    #[error("No hwmod has been found in sysfs")]
    NoHwMonFound,
    #[error("No AMD Card has been found in sysfs")]
    NoAmdCardFound,
    #[error("{0}")]
    AmdGpu(#[from] AmdGpuError),
    #[error("{0}")]
    Fan(#[from] FanError),
    #[error("{0}")]
    Config(#[from] ConfigError),
    #[error("{0:}")]
    Io(#[from] std::io::Error),
}
