use gumdrop::Options;

use amdgpu::utils::ensure_config_dir;
use amdgpu_config::voltage::{Config, load_config};

use crate::command::VoltageCommand;
use crate::error::VoltageError;

mod apply_changes;
mod change_state;
mod clock_state;
mod command;
mod error;
mod print_states;
mod setup_info;

pub static DEFAULT_CONFIG_PATH: &str = "/etc/amdfand/voltage.toml";

pub type Result<T> = std::result::Result<T, VoltageError>;

#[derive(gumdrop::Options)]
pub struct Opts {
    #[options(help = "Help message")]
    help: bool,
    #[options(help = "Print version")]
    version: bool,
    #[options(help = "Config location")]
    config: Option<String>,
    #[options(command)]
    command: Option<command::VoltageCommand>,
}

fn run(config: Config) -> Result<()> {
    let opts: Opts = Opts::parse_args_default_or_exit();

    if opts.version {
        println!("amdfand {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    match opts.command {
        None => {
            Opts::usage();
            Ok(())
        }
        Some(VoltageCommand::PrintStates(command)) => print_states::run(command, config),
        Some(VoltageCommand::SetupInfo(command)) => setup_info::run(command, &config),
        Some(VoltageCommand::ChangeState(command)) => change_state::run(command, &config),
        Some(VoltageCommand::ApplyChanges(command)) => apply_changes::run(command, &config),
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
        .unwrap_or_else(|| DEFAULT_CONFIG_PATH.to_string());
    let config = load_config(&config_path)?;
    log::set_max_level(config.log_level().as_str().parse().unwrap());
    Ok((config_path, config))
}

fn main() -> Result<()> {
    let (config_path, config) = match setup() {
        Ok(config) => config,
        Err(e) => {
            log::error!("{}", e);
            std::process::exit(1);
        }
    };
    match run(config) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _config = load_config(&config_path).expect(
                "Unable to restore automatic voltage control due to unreadable config file",
            );
            // panic_handler::restore_automatic(config);
            log::error!("{}", e);
            std::process::exit(1);
        }
    }
}
