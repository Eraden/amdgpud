use amdgpu::{utils, AmdGpuError};
use amdgpu_config::voltage::ConfigError;

use crate::change_state::ChangeStateError;
use crate::clock_state::ClockStateError;

#[derive(Debug, thiserror::Error)]
pub enum VoltageError {
    #[error("No AMD GPU card was found")]
    NoAmdGpu,
    #[error("Unknown hardware module {0:?}")]
    UnknownHardwareModule(String),
    #[error("{0}")]
    AmdGpu(AmdGpuError),
    #[error("{0}")]
    Config(#[from] ConfigError),
    #[error("{0:}")]
    Io(#[from] std::io::Error),
    #[error("{0:}")]
    ClockState(#[from] ClockStateError),
    #[error("{0:}")]
    ChangeStateError(#[from] ChangeStateError),
    #[error("{0:}")]
    AmdUtils(#[from] utils::AmdGpuError),
}
