mod command;
mod log_file;
mod watch;

use gumdrop::Options;

use crate::command::Command;
use amdgpu::utils::ensure_config_dir;
use amdgpu_config::monitor::{load_config, Config, DEFAULT_MONITOR_CONFIG_PATH};

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

fn run(config: Config) -> amdmond_lib::Result<()> {
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

fn setup() -> amdmond_lib::Result<(String, Config)> {
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

fn main() -> amdmond_lib::Result<()> {
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
