#[cfg(feature = "gui-helper")]
use crate::helper_cmd::GuiHelperError;
use pidlock::PidlockError;
use std::fmt::{Debug, Display, Formatter};

pub struct IoFailure {
    pub io: std::io::Error,
    pub path: std::path::PathBuf,
}

impl std::error::Error for IoFailure {}

impl Debug for IoFailure {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "File system error for {:?}. {:?}",
            self.path, self.io
        ))
    }
}

impl Display for IoFailure {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AmdGpuError {
    #[error("Card must starts with `card`.")]
    CardInvalidPrefix,
    #[error("Card must start with `card` and ends with a number. The given name is too short.")]
    CardInputTooShort,
    #[error("Value after `card` is invalid {0:}")]
    CardInvalidSuffix(String),
    #[error("Invalid temperature input")]
    InvalidTempInput(String),
    #[error("Unable to read GPU vendor")]
    FailedReadVendor,
    #[error("{0}")]
    Io(#[from] IoFailure),
    #[error("Card does not have hwmon")]
    NoAmdHwMon,
    #[error("{0:?}")]
    PidFile(#[from] PidLockError),
    #[cfg(feature = "gui-helper")]
    #[error("{0:?}")]
    GuiHelper(#[from] GuiHelperError),
    #[error("{0:?}")]
    LockFile(#[from] LockFileError),
}

#[derive(Debug, thiserror::Error)]
pub enum PidLockError {
    #[error("A lock already exists")]
    LockExists,
    #[error("An operation was attempted in the wrong state, e.g. releasing before acquiring.")]
    InvalidState,
}

impl From<pidlock::PidlockError> for PidLockError {
    fn from(e: PidlockError) -> Self {
        match e {
            pidlock::PidlockError::LockExists => PidLockError::LockExists,
            pidlock::PidlockError::InvalidState => PidLockError::InvalidState,
        }
    }
}

impl PartialEq for AmdGpuError {
    fn eq(&self, other: &Self) -> bool {
        use AmdGpuError::*;
        match (self, other) {
            (CardInvalidPrefix, CardInvalidPrefix) => true,
            (CardInputTooShort, CardInputTooShort) => true,
            (CardInvalidSuffix(a), CardInvalidSuffix(b)) => a == b,
            (InvalidTempInput(a), InvalidTempInput(b)) => a == b,
            (FailedReadVendor, FailedReadVendor) => true,
            (NoAmdHwMon, NoAmdHwMon) => true,
            (Io(a), Io(b)) => a.io.kind() == b.io.kind(),
            _ => false,
        }
    }
}
