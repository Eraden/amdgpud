#[cfg(feature = "fan")]
pub mod fan;
#[cfg(feature = "monitor")]
pub mod monitor;
#[cfg(feature = "voltage")]
pub mod voltage;

/// pulse width modulation fan level (0-255)
pub static PULSE_WIDTH_MODULATION: &str = "pwm1";
