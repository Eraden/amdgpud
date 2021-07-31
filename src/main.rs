mod config;

extern crate log;

use std::io::Error as IoErr;
use std::io::ErrorKind;

use crate::config::{load_config, Card, Config};
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

/// pulse width modulation fan control minimum level (0)
static PULSE_WIDTH_MODULATION_MIN: &str = "pwm1_min";

/// pulse width modulation fan control maximum level (255)
static PULSE_WIDTH_MODULATION_MAX: &str = "pwm1_max";

/// pulse width modulation fan level (0-255)
static PULSE_WIDTH_MODULATION: &str = "pwm1";

/// pulse width modulation fan control method (0: no fan speed control, 1: manual fan speed control using pwm interface, 2: automatic fan speed control)
static PULSE_WIDTH_MODULATION_ENABLED: &str = "pwm1_enable";

mod hw_mon {
    use std::io::Error as IoErr;

    use crate::config::Card;
    use crate::{
        linear_map, AmdFanError, HwMon, HW_MON_DIR, PULSE_WIDTH_MODULATION,
        PULSE_WIDTH_MODULATION_ENABLED, PULSE_WIDTH_MODULATION_MAX, PULSE_WIDTH_MODULATION_MIN,
        ROOT_DIR,
    };
    use std::io::ErrorKind;

    impl HwMon {
        pub fn new(card: &Card, name: &str) -> Self {
            Self {
                card: card.clone(),
                name: String::from(name),
                pwm_min: None,
                pwm_max: None,
                temp_inputs: load_temp_inputs(&card, name),
            }
        }

        pub fn max_gpu_temp(&self) -> std::io::Result<f64> {
            let mut results = Vec::with_capacity(self.temp_inputs.len());
            for name in self.temp_inputs.iter() {
                results.push(self.read_gpu_temp(name).unwrap_or(0));
            }
            results.sort();
            results
                .last()
                .copied()
                .map(|temp| temp as f64 / 1000f64)
                .ok_or_else(|| IoErr::from(ErrorKind::InvalidInput))
        }

        pub fn gpu_temp(&self) -> Vec<(String, std::io::Result<f64>)> {
            self.temp_inputs
                .clone()
                .into_iter()
                .map(|name| {
                    let temp = self
                        .read_gpu_temp(name.as_str())
                        .map(|temp| temp as f64 / 1000f64);
                    (name, temp)
                })
                .collect()
        }

        fn read_gpu_temp(&self, name: &str) -> std::io::Result<u64> {
            self.read(name)?.parse::<u64>().map_err(|e| {
                log::warn!("Read from gpu monitor failed. Invalid temperature. {}", e);
                IoErr::from(ErrorKind::InvalidInput)
            })
        }

        #[inline]
        pub(crate) fn name(&self) -> std::io::Result<String> {
            self.read("name")
        }

        pub fn pwm_min(&mut self) -> u32 {
            if self.pwm_min.is_none() {
                self.pwm_min = Some(self.value_or_fallback(PULSE_WIDTH_MODULATION_MIN, 0));
            };
            self.pwm_min.unwrap_or_default()
        }

        pub fn pwm_max(&mut self) -> u32 {
            if self.pwm_max.is_none() {
                self.pwm_max = Some(self.value_or_fallback(PULSE_WIDTH_MODULATION_MAX, 255));
            };
            self.pwm_max.unwrap_or(255)
        }

        pub fn pwm(&self) -> std::io::Result<u32> {
            self.read(PULSE_WIDTH_MODULATION)?.parse().map_err(|_e| {
                log::warn!("Read from gpu monitor failed. Invalid pwm value");
                IoErr::from(ErrorKind::InvalidInput)
            })
        }

        pub fn is_fan_manual(&self) -> bool {
            self.read(PULSE_WIDTH_MODULATION_ENABLED)
                .map(|s| s.as_str() == "1")
                .unwrap_or_default()
        }

        pub fn is_fan_automatic(&self) -> bool {
            self.read(PULSE_WIDTH_MODULATION_ENABLED)
                .map(|s| s.as_str() == "2")
                .unwrap_or_default()
        }

        #[inline]
        pub fn is_amd(&self) -> bool {
            std::fs::read_to_string(format!("{}/{}/device/vendor", ROOT_DIR, self.card))
                .map_err(|_| AmdFanError::FailedReadVendor)
                .map(|vendor| vendor.trim() == "0x1002")
                .unwrap_or_default()
        }

        #[inline]
        pub fn name_is_amd(&self) -> bool {
            self.name().ok().filter(|s| s.trim() == "amdgpu").is_some()
        }

        pub fn set_manual(&self) -> std::io::Result<()> {
            self.write("pwm1_enable", 1)
        }

        pub fn set_automatic(&self) -> std::io::Result<()> {
            self.write("pwm1_enable", 2)
        }

        pub fn set_pwm(&self, value: u32) -> std::io::Result<()> {
            if self.is_fan_automatic() {
                self.set_manual()?;
            }
            self.write("pwm1", value as u64)
        }

        pub fn set_speed(&mut self, speed: f64) -> std::io::Result<()> {
            let min = self.pwm_min() as f64;
            let max = self.pwm_max() as f64;
            let pwm = linear_map(speed, 0f64, 100f64, min, max).round() as u32;
            self.set_pwm(pwm)
        }

        fn read(&self, name: &str) -> std::io::Result<String> {
            std::fs::read_to_string(self.path(name)).map(|s| String::from(s.trim()))
        }

        fn write(&self, name: &str, value: u64) -> std::io::Result<()> {
            std::fs::write(self.path(name), format!("{}", value))?;
            Ok(())
        }

        fn path(&self, name: &str) -> std::path::PathBuf {
            self.mon_dir().join(name)
        }

        fn mon_dir(&self) -> std::path::PathBuf {
            hw_mon_dir_path(&self.card, &self.name)
        }

        #[inline]
        fn value_or_fallback<R: std::str::FromStr>(&self, name: &str, fallback: R) -> R {
            self.read(name)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(fallback)
        }
    }

    fn load_temp_inputs(card: &Card, name: &str) -> Vec<String> {
        let dir = match std::fs::read_dir(hw_mon_dir_path(card, name)) {
            Ok(d) => d,
            _ => return vec![],
        };
        dir.filter_map(|f| f.ok())
            .filter_map(|f| {
                f.file_name()
                    .to_str()
                    .filter(|s| s.starts_with("temp") && s.ends_with("_input"))
                    .map(String::from)
            })
            .collect()
    }

    #[inline]
    fn hw_mon_dirs_path(card: &Card) -> std::path::PathBuf {
        std::path::PathBuf::new()
            .join(ROOT_DIR)
            .join(card.to_string())
            .join(HW_MON_DIR)
    }

    #[inline]
    fn hw_mon_dir_path(card: &Card, name: &str) -> std::path::PathBuf {
        hw_mon_dirs_path(&card).join(name)
    }

    pub(crate) fn open_hw_mon(card: Card) -> std::io::Result<HwMon> {
        let read_path = hw_mon_dirs_path(&card);
        let entries = std::fs::read_dir(read_path)?;
        let name = entries
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                entry
                    .file_name()
                    .as_os_str()
                    .to_str()
                    .filter(|name| name.starts_with("hwmon"))
                    .map(String::from)
            })
            .take(1)
            .last()
            .ok_or_else(|| IoErr::from(ErrorKind::NotFound))?;
        Ok(HwMon::new(&card, &name))
    }
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

fn invalid_data() -> IoErr {
    IoErr::from(ErrorKind::InvalidData)
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
    let mut cache = std::collections::HashMap::new();
    loop {
        for hw_mon in controllers.iter_mut() {
            let gpu_temp = hw_mon.max_gpu_temp().unwrap_or_default();
            log::info!("Current {} temperature: {}", hw_mon.card, gpu_temp);
            let last = *cache.entry(*hw_mon.card).or_insert(1_000f64);

            if ((last - 0.001f64)..(last + 0.001f64)).contains(&gpu_temp) {
                log::info!("Temperature didn't change");
                continue;
            };
            let speed = config.speed_for_temp(gpu_temp);
            log::info!("Resolved speed {:.2}", speed);

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
                return Err(IoErr::from(ErrorKind::NotFound));
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

mod monitor {
    use crate::config::Config;
    use crate::{controllers, AmdFanError};
    use std::str::FromStr;

    #[derive(Debug)]
    pub enum MonitorFormat {
        Short,
        Verbose,
    }

    impl Default for MonitorFormat {
        fn default() -> Self {
            MonitorFormat::Short
        }
    }

    impl FromStr for MonitorFormat {
        type Err = AmdFanError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "short" | "s" => Ok(MonitorFormat::Short),
                "verbose" | "v" | "long" | "l" => Ok(MonitorFormat::Verbose),
                _ => Err(AmdFanError::InvalidMonitorFormat),
            }
        }
    }

    #[derive(Debug, gumdrop::Options)]
    pub struct Monitor {
        #[options(help = "Help message")]
        help: bool,
        #[options(help = "Help message")]
        format: MonitorFormat,
    }

    pub fn run(monitor: Monitor, config: Config) -> std::io::Result<()> {
        match monitor.format {
            MonitorFormat::Short => short(config),
            MonitorFormat::Verbose => verbose(config),
        }
    }

    pub fn verbose(config: Config) -> std::io::Result<()> {
        let mut controllers = controllers(&config, true)?;
        loop {
            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
            for hw_mon in controllers.iter_mut() {
                println!("Card {:3}", hw_mon.card.to_string().replace("card", ""));
                println!("  MIN |  MAX |  PWM   |   %");
                println!(
                    " {:>4} | {:>4} | {:>6} | {:>3}",
                    hw_mon.pwm_min(),
                    hw_mon.pwm_max(),
                    hw_mon
                        .pwm()
                        .map_or_else(|_e| String::from("FAILED"), |f| f.to_string()),
                    (hw_mon.pwm().unwrap_or_default() as f32 / 2.55).round(),
                );

                println!();
                println!("  Current temperature");
                hw_mon.gpu_temp().into_iter().for_each(|(name, temp)| {
                    println!(
                        "  {:6} | {:>9.2}",
                        name.replace("_input", ""),
                        temp.unwrap_or_default(),
                    );
                });
            }
            println!();
            println!("> PWM may be 0 even if RPM is higher");
            std::thread::sleep(std::time::Duration::from_secs(4));
        }
    }

    pub fn short(config: Config) -> std::io::Result<()> {
        let mut controllers = controllers(&config, true)?;
        loop {
            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
            for hw_mon in controllers.iter_mut() {
                println!(
                    "Card {:3} | Temp     |  MIN |  MAX |  PWM |   %",
                    hw_mon.card.to_string().replace("card", "")
                );
                println!(
                    "         | {:>5.2}    | {:>4} | {:>4} | {:>4} | {:>3}",
                    hw_mon.max_gpu_temp().unwrap_or_default(),
                    hw_mon.pwm_min(),
                    hw_mon.pwm_max(),
                    hw_mon.pwm().unwrap_or_default(),
                    (hw_mon.pwm().unwrap_or_default() as f32 / 2.55).round(),
                );
            }
            std::thread::sleep(std::time::Duration::from_secs(4));
        }
    }
}

fn main() -> std::io::Result<()> {
    if std::fs::read(CONFIG_DIR).map_err(|e| e.kind() == ErrorKind::NotFound) == Err(true) {
        std::fs::create_dir_all(CONFIG_DIR)?;
    }

    let config = load_config()?;

    std::env::set_var("RUST_LOG", config.log_level().to_str());
    pretty_env_logger::init();

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
