mod config;

extern crate log;

use std::io::ErrorKind;

use crate::config::{load_config, Card, Config};
use gumdrop::Options;

static CONFIG_PATH: &str = "/etc/amdfand/config.toml";

static ROOT_DIR: &str = "/sys/class/drm";
static HW_MON_DIR: &str = "device/hwmon";

pub enum AmdFanError {
    InvalidPrefix,
    InputTooShort,
    InvalidSuffix(String),
    NotAmdCard,
    FailedReadVendor,
}

#[derive(Debug)]
pub struct HwMon {
    card: Card,
    name: String,
    fan_min: Option<u32>,
    fan_max: Option<u32>,
}

impl HwMon {
    pub fn new(card: &Card, name: &str) -> Self {
        Self {
            card: card.clone(),
            name: String::from(name),
            fan_min: None,
            fan_max: None,
        }
    }

    pub fn gpu_temp(&self) -> std::io::Result<f64> {
        let value = self.read("temp1_input")?.parse::<u64>().map_err(|_| {
            log::warn!("Read from gpu monitor failed. Invalid temperature");
            std::io::Error::from(ErrorKind::InvalidInput)
        })?;
        Ok(value as f64 / 1000f64)
    }

    fn name(&self) -> std::io::Result<String> {
        self.read("name")
    }

    pub fn fan_min(&mut self) -> u32 {
        if self.fan_min.is_none() {
            self.fan_min = Some(
                self.read("fan1_min")
                    .unwrap_or_default()
                    .parse()
                    .unwrap_or(0),
            )
        };
        self.fan_min.unwrap_or(0)
    }

    pub fn fan_max(&mut self) -> u32 {
        if self.fan_max.is_none() {
            self.fan_max = Some(
                self.read("fan1_max")
                    .unwrap_or_default()
                    .parse()
                    .unwrap_or(255),
            )
        };
        self.fan_max.unwrap_or(255)
    }

    pub fn fan_speed(&self) -> std::io::Result<u64> {
        self.read("fan1_input")?.parse().map_err(|_e| {
            log::warn!("Read from gpu monitor failed. Invalid fan speed");
            std::io::Error::from(ErrorKind::InvalidInput)
        })
    }

    pub fn is_fan_manual(&self) -> bool {
        self.read("pwm1_enable")
            .map(|s| s.as_str() == "1")
            .unwrap_or_default()
    }

    pub fn is_fan_automatic(&self) -> bool {
        self.read("pwm1_enable")
            .map(|s| s.as_str() == "2")
            .unwrap_or_default()
    }

    pub fn set_manual(&self) -> std::io::Result<()> {
        self.write("pwm1_enable", 1)
    }

    pub fn set_automatic(&self) -> std::io::Result<()> {
        self.write("pwm1_enable", 2)
    }

    pub fn set_speed(&self, speed: u64) -> std::io::Result<()> {
        if self.is_fan_automatic() {
            self.set_manual()?;
        }
        self.write("pwm1", speed)
    }

    fn read(&self, name: &str) -> std::io::Result<String> {
        let read_path = self.path(name);
        std::fs::read_to_string(read_path).map(|s| s.trim().to_string())
    }

    fn write(&self, name: &str, value: u64) -> std::io::Result<()> {
        std::fs::write(self.path(name), format!("{}", value))?;
        Ok(())
    }

    fn path(&self, name: &str) -> std::path::PathBuf {
        self.mon_dir().join(name)
    }

    fn mon_dir(&self) -> std::path::PathBuf {
        std::path::Path::new(ROOT_DIR)
            .join(self.card.to_string())
            .join(HW_MON_DIR)
            .join(&self.name)
    }
}

pub struct CardController {
    pub hw_mon: HwMon,
    pub last_temp: f64,
}

impl CardController {
    pub fn new(card: Card) -> std::io::Result<Self> {
        let name = Self::find_hw_mon(&card)?;
        Ok(Self {
            hw_mon: HwMon::new(&card, &name),
            last_temp: 1_000f64,
        })
    }

    fn find_hw_mon(card: &Card) -> std::io::Result<String> {
        let read_path = format!("{}/{}/{}", ROOT_DIR, card.to_string(), HW_MON_DIR);
        let entries = std::fs::read_dir(read_path)?;
        entries
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                entry
                    .file_name()
                    .as_os_str()
                    .to_str()
                    .map(|s| s.to_string())
            })
            .find(|name| name.starts_with("hwmon"))
            .ok_or_else(|| std::io::Error::from(ErrorKind::NotFound))
    }
}

pub enum FanMode {
    Manual,
    Automatic,
}

#[derive(Debug, Options)]
pub struct Monitor {
    #[options(help = "Help message")]
    help: bool,
}

#[derive(Debug, Options)]
pub struct Service {
    #[options(help = "Help message")]
    help: bool,
}

#[derive(Debug, Options)]
pub struct Switcher {
    #[options(help = "Help message")]
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
    Monitor(Monitor),
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
                    .map(|s| s.to_string())
                    .ok_or_else(|| std::io::Error::from(ErrorKind::InvalidData))
            })
            .and_then(|file_name| {
                file_name
                    .parse::<Card>()
                    .map_err(|_| std::io::Error::from(ErrorKind::InvalidData))
            }) {
            Ok(card) => {
                cards.push(card);
            }
            _ => continue,
        };
    }
    Ok(cards)
}

fn controllers(config: &Config, filter: bool) -> std::io::Result<Vec<CardController>> {
    Ok(read_cards()?
        .into_iter()
        .filter(|card| {
            !filter
                || config
                    .cards()
                    .iter()
                    .find(|name| name.0 == card.0)
                    .is_some()
        })
        .map(|card| CardController::new(card).unwrap())
        .filter(|reader| {
            !filter
                || reader
                    .hw_mon
                    .name()
                    .ok()
                    .filter(|s| s.as_str() == "amdgpu")
                    .is_some()
        })
        .collect())
}

fn service(config: Config) -> std::io::Result<()> {
    let mut controllers = controllers(&config, true)?;
    loop {
        for controller in controllers.iter_mut() {
            let gpu_temp = controller.hw_mon.gpu_temp().unwrap_or_default();

            let speed = config.speed_for_temp(gpu_temp);
            if controller.hw_mon.fan_min() > speed || controller.hw_mon.fan_max() < speed {
                continue;
            }

            if let Err(e) = controller.hw_mon.set_speed(speed as u64) {
                log::error!("Failed to change speed to {}. {:?}", speed, e);
            }
            controller.last_temp = gpu_temp;
        }
        std::thread::sleep(std::time::Duration::from_secs(4));
    }
}

fn change_mode(switcher: Switcher, mode: FanMode, config: Config) -> std::io::Result<()> {
    let mut controllers = controllers(&config, true)?;

    let cards = match switcher.card {
        Some(card_id) => match controllers.iter().position(|c| c.hw_mon.card.0 == card_id) {
            Some(card) => vec![controllers.remove(card)],
            None => {
                eprintln!("Card does not exists. Available cards: ");
                for card in controllers {
                    eprintln!(" * {}", card.hw_mon.card.0);
                }
                return Err(std::io::Error::from(ErrorKind::NotFound));
            }
        },
        None => controllers,
    };

    for card in cards {
        match mode {
            FanMode::Automatic => {
                if let Err(e) = card.hw_mon.set_automatic() {
                    log::error!("{:?}", e);
                }
            }
            FanMode::Manual => {
                if let Err(e) = card.hw_mon.set_manual() {
                    log::error!("{:?}", e);
                }
            }
        }
    }
    Ok(())
}

fn monitor_cards(config: Config) -> std::io::Result<()> {
    let mut controllers = controllers(&config, true)?;
    loop {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        for card in controllers.iter_mut() {
            println!(
                "Card {:3} | Temp     | fan speed |  MAX |  MIN ",
                card.hw_mon.card.0
            );
            println!(
                "         | {:>5.2}    | {:>9} | {:>4} | {:>4}",
                card.hw_mon.gpu_temp().unwrap_or_default(),
                card.hw_mon.fan_speed().unwrap_or_default(),
                card.hw_mon.fan_min(),
                card.hw_mon.fan_max(),
            );
        }
        std::thread::sleep(std::time::Duration::from_secs(4));
    }
}

fn main() -> std::io::Result<()> {
    if std::fs::read("/etc/amdfand").map_err(|e| e.kind() == ErrorKind::NotFound) == Err(true) {
        std::fs::create_dir_all("/etc/amdfand")?;
    }

    let config = load_config()?;

    log::set_max_level(config.log_level().into());
    pretty_env_logger::init();

    let opts: Opts = Opts::parse_args_default_or_exit();

    match opts.command {
        None => return Ok(()),
        Some(Command::Monitor(_)) => {
            monitor_cards(config)?;
        }
        Some(Command::Service(_)) => {
            service(config)?;
        }
        Some(Command::SetAutomatic(switcher)) => {
            change_mode(switcher, FanMode::Automatic, config)?;
        }
        Some(Command::SetManual(switcher)) => {
            change_mode(switcher, FanMode::Manual, config)?;
        }
        Some(Command::Available(_)) => {
            println!("Available cards");
            controllers(&config, false)?.into_iter().for_each(|card| {
                println!(
                    " * {:6>} - {}",
                    card.hw_mon.card.to_string(),
                    card.hw_mon.name().unwrap_or_default()
                );
            });
        }
    }

    Ok(())
}
