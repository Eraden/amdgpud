use std::fmt::Formatter;
use std::io::ErrorKind;

use gumdrop::Options;

use crate::config::{load_config, Card};

mod config;
mod fan;
mod hw_mon;
mod io_err;
mod utils;
mod voltage;

extern crate log;

static CONFIG_DIR: &str = "/etc/amdfand";
static CONFIG_PATH: &str = "/etc/amdfand/config.toml";

static ROOT_DIR: &str = "/sys/class/drm";
static HW_MON_DIR: &str = "device/hwmon";

#[derive(Debug, PartialEq)]
pub enum AmdFanError {
    InvalidPrefix,
    InputTooShort,
    InvalidSuffix(String),
    NotAmdCard,
    FailedReadVendor,
    InvalidMonitorFormat,
}

impl std::fmt::Display for AmdFanError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AmdFanError::InvalidPrefix => f.write_str("Card must starts with `card`."),
            AmdFanError::InputTooShort => f.write_str(
                "Card must start with `card` and ends with a number. The given name is too short.",
            ),
            AmdFanError::InvalidSuffix(s) => {
                f.write_fmt(format_args!("Value after `card` is invalid {}", s))
            }
            AmdFanError::NotAmdCard => f.write_str("Vendor is not AMD"),
            AmdFanError::FailedReadVendor => f.write_str("Unable to read GPU vendor"),
            AmdFanError::InvalidMonitorFormat => f.write_str("Monitor format is not valid. Available values are: short, s, long l, verbose and v"),
        }
    }
}

#[derive(Debug)]
pub struct HwMon {
    card: Card,
    name: String,
    pwm_min: Option<u32>,
    pwm_max: Option<u32>,
    temp_inputs: Vec<String>,
}

pub enum FanMode {
    Manual,
    Automatic,
}

#[derive(Options)]
pub enum Command {
    #[options(help = "GPU card fan control")]
    Fan(fan::FanCommand),
    #[options(help = "Overclock GPU card")]
    Voltage(voltage::VoltageCommand),
}

#[derive(Options)]
pub struct Opts {
    #[options(help = "Help message")]
    help: bool,
    #[options(help = "Print version")]
    version: bool,
    #[options(command)]
    command: Option<Command>,
}

fn main() -> std::io::Result<()> {
    if std::fs::read(CONFIG_DIR).map_err(|e| e.kind() == ErrorKind::NotFound) == Err(true) {
        std::fs::create_dir_all(CONFIG_DIR)?;
    }

    let config = load_config()?;

    std::env::set_var("RUST_LOG", config.log_level().as_str());
    pretty_env_logger::init();

    let opts: Opts = Opts::parse_args_default_or_exit();

    if opts.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    match opts.command {
        None => fan::service::run(config),
        Some(Command::Fan(fan::FanCommand::Monitor(monitor))) => fan::monitor::run(monitor, config),
        Some(Command::Fan(fan::FanCommand::Service(_))) => fan::service::run(config),
        Some(Command::Fan(fan::FanCommand::SetAutomatic(switcher))) => {
            fan::change_mode::run(switcher, FanMode::Automatic, config)
        }
        Some(Command::Fan(fan::FanCommand::SetManual(switcher))) => {
            fan::change_mode::run(switcher, FanMode::Manual, config)
        }
        Some(Command::Fan(fan::FanCommand::Available(_))) => {
            println!("Available cards");
            utils::controllers(&config, false)?
                .into_iter()
                .for_each(|hw_mon| {
                    println!(
                        " * {:6>} - {}",
                        hw_mon.card,
                        hw_mon.name().unwrap_or_default()
                    );
                });
            Ok(())
        }
        Some(Command::Voltage(voltage::VoltageCommand::Placeholder(_))) => {
            unimplemented!()
        }
    }
}
