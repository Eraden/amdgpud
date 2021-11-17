use crate::command::FanCommand;
use crate::error::AmdFanError;
use amdgpu::utils::hw_mons;
use amdgpu::CONFIG_DIR;
use config::{load_config, Config};
use gumdrop::Options;
use std::io::ErrorKind;

mod change_mode;
mod command;
mod config;
mod error;
mod monitor;
mod panic_handler;
mod service;

extern crate log;

pub static DEFAULT_CONFIG_PATH: &str = "/etc/amdfand/config.toml";

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
    #[options(command)]
    command: Option<command::FanCommand>,
}

fn run(config: Config) -> Result<()> {
    let opts: Opts = Opts::parse_args_default_or_exit();

    if opts.version {
        println!("amdfand {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }
    #[allow(deprecated)]
    if config.cards().is_some() {
        log::warn!("cards config field is no longer supported");
    }

    match opts.command {
        None => service::run(config),
        Some(FanCommand::Monitor(monitor)) => monitor::run(monitor, config),
        Some(FanCommand::Service(_)) => service::run(config),
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

fn setup() -> Result<(String, Config)> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "DEBUG");
    }
    pretty_env_logger::init();
    if std::fs::read(CONFIG_DIR).map_err(|e| e.kind() == ErrorKind::NotFound) == Err(true) {
        std::fs::create_dir_all(CONFIG_DIR)?;
    }

    let config_path = Opts::parse_args_default_or_exit()
        .config
        .unwrap_or_else(|| DEFAULT_CONFIG_PATH.to_string());
    let config = load_config(&config_path)?;
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
            panic_handler::restore_automatic();
            log::error!("{}", e);
            std::process::exit(1);
        }
    }
}
