mod command;
mod log_file;
mod watch;

use amdgpu::utils;
use gumdrop::Options;

use crate::command::Command;
use amdgpu::utils::ensure_config_dir;
use amdgpu_config::monitor::{load_config, Config, DEFAULT_MONITOR_CONFIG_PATH};
use amdgpu_config::{fan, monitor};

#[derive(Debug, thiserror::Error)]
pub enum AmdMonError {
    #[error("Mon AMD GPU card was found")]
    NoHwMon,
    #[error("{0}")]
    Io(#[from] std::io::Error),
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

pub type Result<T> = std::result::Result<T, AmdMonError>;

#[derive(gumdrop::Options)]
pub struct Opts {
    #[options(help = "Help message")]
    pub help: bool,
    #[options(help = "Print version")]
    pub version: bool,
    #[options(help = "Config location")]
    pub config: Option<String>,
    #[options(command)]
    pub command: Option<Command>,
}

fn run(config: Config) -> Result<()> {
    let opts: Opts = Opts::parse_args_default_or_exit();

    if opts.version {
        println!("amdfand {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }
    match opts.command {
        Some(Command::Watch(w)) => watch::run(w, config),
        Some(Command::LogFile(l)) => log_file::run(l, config),
        _ => {
            println!("{}", <Opts as gumdrop::Options>::usage());
            Ok(())
        }
    }
}

fn setup() -> Result<(String, Config)> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "DEBUG");
    }
    pretty_env_logger::init();
    ensure_config_dir()?;

    let config_path = Opts::parse_args_default_or_exit()
        .config
        .unwrap_or_else(|| DEFAULT_MONITOR_CONFIG_PATH.to_string());
    let config = load_config(&config_path)?;
    log::info!("{:?}", config);
    log::set_max_level(config.log_level().as_str().parse().unwrap());
    Ok((config_path, config))
}

fn main() -> Result<()> {
    let (_config_path, config) = match setup() {
        Ok(config) => config,
        Err(e) => {
            log::error!("{}", e);
            std::process::exit(1);
        }
    };
    match run(config) {
        Ok(()) => Ok(()),
        Err(e) => {
            log::error!("{}", e);
            std::process::exit(1);
        }
    }
}
