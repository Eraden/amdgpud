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
    #[error("{0:?}")]
    Io(#[from] std::io::Error),
    #[error("Card does not have hwmon")]
    NoAmdHwMon,
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
            (Io(a), Io(b)) => a.kind() == b.kind(),
            _ => false,
        }
    }
}
