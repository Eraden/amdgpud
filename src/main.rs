mod config;
mod hw_mon;
mod io_err;
mod monitor;

extern crate log;

use std::io::ErrorKind;

use crate::config::{load_config, Card, Config};
use crate::io_err::{invalid_data, not_found};
use gumdrop::Options;
use std::fmt::Formatter;

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

// linear mapping from the xrange to the yrange
fn linear_map(x: f64, x1: f64, x2: f64, y1: f64, y2: f64) -> f64 {
    let m = (y2 - y1) / (x2 - x1);
    m * (x - x1) + y1
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

#[derive(Debug, Options)]
pub struct Service {
    #[options(help = "Help message")]
    help: bool,
}

#[derive(Debug, Options)]
pub struct Switcher {
    #[options(help = "Print help message")]
    help: bool,
    #[options(help = "GPU Card number")]
    card: Option<u32>,
}

#[derive(Debug, Options)]
pub struct AvailableCards {
    #[options(help = "Help message")]
    help: bool,
}

#[derive(Debug, Options)]
pub enum Command {
    #[options(help = "Print current temp and fan speed")]
    Monitor(monitor::Monitor),
    #[options(help = "Set fan speed depends on GPU temperature")]
    Service(Service),
    #[options(help = "Switch to GPU automatic fan speed control")]
    SetAutomatic(Switcher),
    #[options(help = "Switch to GPU manual fan speed control")]
    SetManual(Switcher),
    #[options(help = "Print available cards")]
    Available(AvailableCards),
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

fn read_cards() -> std::io::Result<Vec<Card>> {
    let mut cards = vec![];
    let entries = std::fs::read_dir(ROOT_DIR)?;
    for entry in entries {
        match entry
            .and_then(|entry| {
                entry
                    .file_name()
                    .as_os_str()
                    .to_str()
                    .map(String::from)
                    .ok_or_else(invalid_data)
            })
            .and_then(|file_name| file_name.parse::<Card>().map_err(|_| invalid_data()))
        {
            Ok(card) => {
                cards.push(card);
            }
            _ => continue,
        };
    }
    Ok(cards)
}

fn controllers(config: &Config, filter: bool) -> std::io::Result<Vec<HwMon>> {
    Ok(read_cards()?
        .into_iter()
        .filter(|card| !filter || config.cards().iter().find(|name| **name == *card).is_some())
        .filter_map(|card| hw_mon::open_hw_mon(card).ok())
        .filter(|hw_mon| !filter || { hw_mon.is_amd() })
        .filter(|hw_mon| !filter || hw_mon.name_is_amd())
        .collect())
}

fn service(config: Config) -> std::io::Result<()> {
    let mut controllers = controllers(&config, true)?;
    if controllers.is_empty() {
        return Err(not_found());
    }
    let mut cache = std::collections::HashMap::new();
    loop {
        for hw_mon in controllers.iter_mut() {
            let gpu_temp = hw_mon.max_gpu_temp().unwrap_or_default();
            log::debug!("Current {} temperature: {}", hw_mon.card, gpu_temp);
            let last = *cache.entry(*hw_mon.card).or_insert(1_000f64);

            if (last - gpu_temp).abs() < 0.001f64 {
                log::debug!("Temperature didn't change");
                continue;
            };
            let speed = config.speed_for_temp(gpu_temp);
            log::debug!("Resolved speed {:.2}", speed);

            if let Err(e) = hw_mon.set_speed(speed) {
                log::error!("Failed to change speed to {}. {:?}", speed, e);
            }
            cache.insert(*hw_mon.card, gpu_temp);
        }
        std::thread::sleep(std::time::Duration::from_secs(4));
    }
}

fn change_mode(switcher: Switcher, mode: FanMode, config: Config) -> std::io::Result<()> {
    let mut controllers = controllers(&config, true)?;

    let cards = match switcher.card {
        Some(card_id) => match controllers
            .iter()
            .position(|hw_mon| *hw_mon.card == card_id)
        {
            Some(card) => vec![controllers.remove(card)],
            None => {
                eprintln!("Card does not exists. Available cards: ");
                for hw_mon in controllers {
                    eprintln!(" * {}", *hw_mon.card);
                }
                return Err(not_found());
            }
        },
        None => controllers,
    };

    for hw_mon in cards {
        match mode {
            FanMode::Automatic => {
                if let Err(e) = hw_mon.set_automatic() {
                    log::error!("{:?}", e);
                }
            }
            FanMode::Manual => {
                if let Err(e) = hw_mon.set_manual() {
                    log::error!("{:?}", e);
                }
            }
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "DEBUG");
    }
    pretty_env_logger::init();
    if std::fs::read(CONFIG_DIR).map_err(|e| e.kind() == ErrorKind::NotFound) == Err(true) {
        std::fs::create_dir_all(CONFIG_DIR)?;
    }

    let config = load_config()?;
    log::set_max_level(config.log_level().to_str().parse().unwrap());

    let opts: Opts = Opts::parse_args_default_or_exit();

    if opts.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    match opts.command {
        None => service(config),
        Some(Command::Monitor(monitor)) => monitor::run(monitor, config),
        Some(Command::Service(_)) => service(config),
        Some(Command::SetAutomatic(switcher)) => change_mode(switcher, FanMode::Automatic, config),
        Some(Command::SetManual(switcher)) => change_mode(switcher, FanMode::Manual, config),
        Some(Command::Available(_)) => {
            println!("Available cards");
            controllers(&config, false)?.into_iter().for_each(|hw_mon| {
                println!(
                    " * {:6>} - {}",
                    hw_mon.card,
                    hw_mon.name().unwrap_or_default()
                );
            });
            Ok(())
        }
    }
}
