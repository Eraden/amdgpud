extern crate log;

use gumdrop::Options;

use amdgpu::utils::{ensure_config_dir, hw_mons};
use amdgpu_config::fan::{load_config, Config, DEFAULT_FAN_CONFIG_PATH};

use crate::command::FanCommand;
use crate::error::AmdFanError;

mod change_mode;
mod command;
mod error;
mod panic_handler;
mod service;

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
    ensure_config_dir()?;

    let config_path = Opts::parse_args_default_or_exit()
        .config
        .unwrap_or_else(|| DEFAULT_FAN_CONFIG_PATH.to_string());
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
            panic_handler::restore_automatic();
            log::error!("{}", e);
            std::process::exit(1);
        }
    }
}
