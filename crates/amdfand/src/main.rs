use amdgpu::lock_file::PidLock;
use amdgpu::utils::{ensure_config_dir, hw_mons};
use amdgpu_config::fan::{load_config, Config, DEFAULT_FAN_CONFIG_PATH};
use gumdrop::Options;
use tracing::level_filters::LevelFilter;
use tracing::{error, warn};
use tracing_subscriber::EnvFilter;

use crate::command::FanCommand;
use crate::error::AmdFanError;

mod change_mode;
mod command;
mod error;
mod panic_handler;
mod service;

#[cfg(feature = "static")]
extern crate eyra;

pub type Result<T> = std::result::Result<T, AmdFanError>;

pub enum FanMode {
    Manual,
    Automatic,
}

#[derive(Options)]
pub struct Opts {
    #[options(help = "Help message")]
    help: bool,
    #[options(help = "Print version")]
    version: bool,
    #[options(help = "Config location")]
    config: Option<String>,
    #[options(
        help = "Pid file name (exp. card1). This should not be path, only file name without extension"
    )]
    pid_file: Option<String>,
    #[options(command)]
    command: Option<command::FanCommand>,
}

static DEFAULT_PID_FILE_NAME: &str = "amdfand";

fn run(config: Config) -> Result<()> {
    let opts: Opts = Opts::parse_args_default_or_exit();

    if opts.version {
        println!("amdfand {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }
    #[allow(deprecated)]
    if config.cards().is_some() {
        warn!("cards config field is no longer supported");
    }

    match opts.command {
        None => run_service(config, opts),
        Some(FanCommand::Service(_)) => run_service(config, opts),
        Some(FanCommand::SetAutomatic(switcher)) => {
            change_mode::run(switcher, FanMode::Automatic, config)
        }
        Some(FanCommand::SetManual(switcher)) => {
            change_mode::run(switcher, FanMode::Manual, config)
        }
        Some(FanCommand::Available(_)) => {
            println!("Available cards");
            hw_mons(false)?.into_iter().for_each(|hw_mon| {
                println!(
                    " * {:6>} - {}",
                    hw_mon.card(),
                    hw_mon.name().unwrap_or_default()
                );
            });
            Ok(())
        }
    }
}

fn run_service(config: Config, opts: Opts) -> Result<()> {
    let mut pid_file = PidLock::new(
        "amdfand",
        opts.pid_file
            .unwrap_or_else(|| String::from(DEFAULT_PID_FILE_NAME)),
    )?;
    pid_file.acquire()?;
    let res = service::run(config);
    pid_file.release()?;
    res
}

fn setup() -> Result<(String, Config)> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "DEBUG");
    }
    ensure_config_dir()?;

    let config_path = Opts::parse_args_default_or_exit()
        .config
        .unwrap_or_else(|| DEFAULT_FAN_CONFIG_PATH.to_string());
    let config = load_config(&config_path)?;
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_max_level(config.log_level().as_str().parse::<LevelFilter>().unwrap())
        .init();
    Ok((config_path, config))
}

fn main() -> Result<()> {
    let (_config_path, config) = match setup() {
        Ok(config) => config,
        Err(e) => {
            error!("{}", e);
            std::process::exit(1);
        }
    };
    match run(config) {
        Ok(()) => Ok(()),
        Err(e) => {
            panic_handler::restore_automatic();
            error!("{}", e);
            std::process::exit(1);
        }
    }
}
