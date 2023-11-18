pub use card::*;
pub use error::*;
use serde::{Deserialize, Serialize};
pub use temp_input::*;

mod card;
mod error;
#[cfg(feature = "gui-helper")]
pub mod hw_mon;
pub mod lock_file;
pub mod pidfile;
mod temp_input;
pub mod utils;

pub static CONFIG_DIR: &str = "/etc/amdfand";

pub static ROOT_DIR: &str = "/sys/class/drm";
pub static HW_MON_DIR: &str = "hwmon";

/// pulse width modulation fan control minimum level (0)
pub static PULSE_WIDTH_MODULATION_MIN: &str = "pwm1_min";

/// pulse width modulation fan control maximum level (255)
pub static PULSE_WIDTH_MODULATION_MAX: &str = "pwm1_max";

/// pulse width modulation fan level (0-255)
pub static PULSE_WIDTH_MODULATION: &str = "pwm1";

/// pulse width modulation fan control method (0: no fan speed control, 1:
/// manual fan speed control using pwm interface, 2: automatic fan speed
/// control)
pub static PULSE_WIDTH_MODULATION_MODE: &str = "pwm1_enable";

// static PULSE_WIDTH_MODULATION_DISABLED: &str = "0";
pub static PULSE_WIDTH_MODULATION_MANUAL: &str = "1";
pub static PULSE_WIDTH_MODULATION_AUTO: &str = "2";

static mut RELOAD_CONFIG: bool = false;

extern "C" fn sig_reload(_n: i32) {
    unsafe {
        RELOAD_CONFIG = true;
    };
}

/// Listen for SIGHUP signal. This signal is used to reload config
pub fn listen_unix_signal() {
    use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal};
    unsafe {
        let handler: SigHandler = SigHandler::Handler(sig_reload);
        let action = SigAction::new(handler, SaFlags::SA_NOCLDWAIT, SigSet::empty());
        sigaction(Signal::SIGHUP, &action).expect("Failed to mount action handler");
    };
}

/// Check if application received SIGHUP and must reload config file
#[inline(always)]
pub fn is_reload_required() -> bool {
    unsafe { RELOAD_CONFIG }
}

/// Reset reload config flag
#[inline(always)]
pub fn config_reloaded() {
    unsafe {
        RELOAD_CONFIG = false;
    }
}

pub type Result<T> = std::result::Result<T, AmdGpuError>;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum LogLevel {
    /// A level lower than all log levels.
    Off,
    /// Corresponds to the `Error` log level.
    Error,
    /// Corresponds to the `Warn` log level.
    Warn,
    /// Corresponds to the `Info` log level.
    Info,
    /// Corresponds to the `Debug` log level.
    Debug,
    /// Corresponds to the `Trace` log level.
    Trace,
}

impl LogLevel {
    pub fn as_str(&self) -> &str {
        match self {
            LogLevel::Off => "OFF",
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }
}
