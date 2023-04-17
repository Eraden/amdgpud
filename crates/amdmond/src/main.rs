mod command;
mod log_file;
mod watch;

use amdgpu::utils::ensure_config_dir;
use amdgpu_config::monitor::{load_config, Config, DEFAULT_MONITOR_CONFIG_PATH};
use gumdrop::Options;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::command::Command;
extern crate eyra;

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
    ensure_config_dir()?;

    let config_path = Opts::parse_args_default_or_exit()
        .config
        .unwrap_or_else(|| DEFAULT_MONITOR_CONFIG_PATH.to_string());
    let config = load_config(&config_path)
        .map_err(|io| amdmond_lib::errors::AmdMonError::Io {
            io: io.into_io(),
            path: config_path.as_str().into(),
        })
        .unwrap();
    println!("{config:?}");

    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_max_level(config.log_level().as_str().parse::<LevelFilter>().unwrap())
        .init();

    Ok((config_path, config))
}

fn main() -> amdmond_lib::Result<()> {
    let (_config_path, config) = match setup() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };
    match run(config) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
