mod config;

extern crate log;

use std::io::ErrorKind;

use crate::config::{load_config, Card, Config, GpuState};
use gumdrop::Options;
use std::str::FromStr;

static CONFIG_PATH: &str = "/etc/amdfand/config.toml";

static ROOT_DIR: &str = "/sys/class/drm";
static HW_MON_DIR: &str = "device/hwmon";

#[derive(Debug)]
pub enum AmdFanError {
    Io(std::io::Error),
    InvalidPrefix,
    InputTooShort,
    InvalidSuffix(String),
    NotAmdCard,
    FailedReadVendor,
    // Profile
    InvalidGpuProfile(String),
    NoCard,
    NoProfile,
    // parse state
    StateNoPosition,
    StateInvalidPosition(String),
    StateMissingColon,
    StateNoFrequency,
    StateInvalidFrequency(String),
    StateNoUnit,
}

impl From<std::io::Error> for AmdFanError {
    fn from(io: std::io::Error) -> Self {
        AmdFanError::Io(io)
    }
}

impl std::fmt::Display for AmdFanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AmdFanError::InvalidPrefix => f.write_str("Card name must starts with \'card\'"),
            AmdFanError::InputTooShort => f.write_str("Card name must have at least 5 characters"),
            AmdFanError::InvalidSuffix(s) => f.write_fmt(format_args!(
                "Card name must have number after \'card\' prefix. {}",
                s.as_str()
            )),
            AmdFanError::NotAmdCard => f.write_str("Unexpected non-AMD card"),
            AmdFanError::FailedReadVendor => f.write_str("Unable to read card vendor"),
            AmdFanError::InvalidGpuProfile(s) => {
                f.write_fmt(format_args!("Unknown GPU profile {}", s))
            }
            AmdFanError::StateNoPosition => {
                f.write_str("State does not have position. Please see documentation")
            }
            AmdFanError::StateInvalidPosition(s) => f.write_fmt(format_args!(
                "State have invalid value for position. Must be a number. {}",
                s
            )),
            AmdFanError::StateMissingColon => {
                f.write_str("State does not contains \':\' after position")
            }
            AmdFanError::StateNoFrequency => f.write_str("State does not have frequency"),
            AmdFanError::StateInvalidFrequency(s) => f.write_fmt(format_args!(
                "State contains invalid value for frequency. {}",
                s
            )),
            AmdFanError::StateNoUnit => {
                f.write_str("State does not have \'Mhz\' after frequency value.")
            }
            AmdFanError::NoCard => {
                f.write_str("You must specify which card which profile you wish to change")
            }
            AmdFanError::NoProfile => f.write_str("You must specify profile you wish to apply!"),
            AmdFanError::Io(io) => f.write_fmt(format_args!("Failed to access io. {}", io)),
        }
    }
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

    pub fn is_amd(&self) -> bool {
        std::fs::read_to_string(format!("{}/{}/device/vendor", ROOT_DIR, self.card))
            .map_err(|_| AmdFanError::FailedReadVendor)
            .map(|vendor| {
                if vendor.trim() == "0x1002" {
                    true
                } else {
                    false
                }
            })
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

    pub fn change_profile(&self, profile: GpuProfile) -> Result<(), AmdFanError> {
        let path = self.device_path().join("power_dpm_force_performance_level");
        std::fs::write(path, profile.to_str())?;
        Ok(())
    }
    pub fn print_states(&self) -> Result<(), AmdFanError> {
        println!("{}", self.hw_mon.card);

        println!("  {:^33}", StateFile::Clock.name());
        println!("  | position | frequency | active |");
        println!("  |----------|-----------|--------|");
        for state in self.read_states(StateFile::Clock)? {
            println!(
                "  | {position:>8} | {frequency:>9} | {active:^6} |",
                position = state.position,
                frequency = state.frequency,
                active = state.active
            );
        }

        println!();

        println!("  {:^33}", StateFile::Clock.name());
        println!("  | position | frequency | active |");
        println!("  |----------|-----------|--------|");
        for state in self.read_states(StateFile::Memory)? {
            println!(
                "  | {position:>8} | {frequency:>9} | {active:^6} |",
                position = state.position,
                frequency = state.frequency,
                active = state.active
            );
        }

        Ok(())
    }

    fn read_states(&self, file: StateFile) -> Result<Vec<GpuState>, AmdFanError> {
        let mut states: Vec<Result<GpuState, AmdFanError>> = self
            .read(file.file_name())?
            .lines()
            .map(|line| line.parse::<GpuState>())
            .collect();
        if let Some(idx) = states.iter().position(|r| r.is_err()) {
            return states.remove(idx).map(|_| vec![]);
        }
        Ok(states.into_iter().map(|r| r.unwrap()).collect())
    }

    fn read(&self, name: &str) -> Result<String, AmdFanError> {
        std::fs::read_to_string(self.device_path().join(name)).map_err(AmdFanError::from)
    }

    fn device_path(&self) -> std::path::PathBuf {
        std::path::Path::new(ROOT_DIR)
            .join(self.hw_mon.card.to_string())
            .join("device")
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

pub enum StateFile {
    /// pp_dpm_sclk
    Clock,
    /// pp_dpm_mclk
    Memory,
}

impl StateFile {
    pub fn file_name(&self) -> &str {
        match self {
            StateFile::Clock => "pp_dpm_sclk",
            StateFile::Memory => "pp_dpm_mclk",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            StateFile::Clock => "Clock frequency",
            StateFile::Memory => "Memory frequency",
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub enum GpuProfile {
    /// Set GPU voltage to automatic
    Auto,
    /// Set GPU voltage to low (under-volt)
    Low,
    /// Set GPU voltage to high (over-volt)
    High,
}

impl FromStr for GpuProfile {
    type Err = AmdFanError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(GpuProfile::Auto),
            "low" => Ok(GpuProfile::Low),
            "high" => Ok(GpuProfile::High),
            _ => Err(AmdFanError::InvalidGpuProfile(String::from(s))),
        }
    }
}

impl GpuProfile {
    pub fn to_str(&self) -> &str {
        match self {
            GpuProfile::Auto => "auto",
            GpuProfile::Low => "low",
            GpuProfile::High => "high",
        }
    }
}

#[derive(Debug, Options)]
pub struct ChangeProfile {
    #[options(help = "Help message")]
    help: bool,
    #[options(help = "GPU Card number", free)]
    card: Option<Card>,
    #[options(help = "Change voltage. Available options: auto, low, high", free)]
    profile: Option<GpuProfile>,
}

#[derive(Debug, Options)]
pub struct StatesPrinter {
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
    #[options(help = "Change GPU profile")]
    Profile(ChangeProfile),
    #[options(help = "print GPU Card states")]
    AvailableStates(StatesPrinter),
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
        .filter(|controller| !filter || controller.hw_mon.is_amd())
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

fn change_profile(profile: ChangeProfile, config: Config) -> Result<(), AmdFanError> {
    let card = profile.card.ok_or_else(|| AmdFanError::NoCard)?;
    let controller = controllers(&config, true)?
        .into_iter()
        .find(|c| c.hw_mon.card == card)
        .expect("Card not found!");
    controller.change_profile(profile.profile.ok_or_else(|| AmdFanError::NoProfile)?)?;
    Ok(())
}

fn print_states(_printer: StatesPrinter, config: Config) -> Result<(), AmdFanError> {
    let cards = controllers(&config, true)?;
    for card in cards {
        card.print_states()?;
    }
    Ok(())
}

fn run() -> Result<(), AmdFanError> {
    if std::fs::read("/etc/amdfand").map_err(|e| e.kind() == ErrorKind::NotFound) == Err(true) {
        std::fs::create_dir_all("/etc/amdfand")?;
    }

    let config = load_config()?;

    log::set_max_level(config.log_level().into());
    pretty_env_logger::init();

    let opts: Opts = Opts::parse_args_default_or_exit();

    if opts.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    match opts.command {
        None => service(config),
        Some(Command::Monitor(_)) => monitor_cards(config),
        Some(Command::Service(_)) => service(config),
        Some(Command::SetAutomatic(switcher)) => change_mode(switcher, FanMode::Automatic, config),
        Some(Command::SetManual(switcher)) => change_mode(switcher, FanMode::Manual, config),
        Some(Command::Available(_)) => {
            println!("Available cards:");
            controllers(&config, false)?.into_iter().for_each(|card| {
                println!(
                    " * {:6>} - {}",
                    card.hw_mon.card.to_string(),
                    card.hw_mon.name().unwrap_or_default()
                );
            });
            Ok(())
        }
        Some(Command::Profile(profile)) => return change_profile(profile, config),
        Some(Command::AvailableStates(printer)) => return print_states(printer, config),
    }?;
    Ok(())
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => {
            log::error!("{}", e);
            std::process::exit(1)
        }
    }
}
