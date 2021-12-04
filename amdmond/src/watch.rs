use std::str::FromStr;

use amdgpu::utils::{hw_mons, linear_map};
use amdgpu_config::fan::DEFAULT_FAN_CONFIG_PATH;
use amdgpu_config::{fan, monitor};
use amdmond_lib::errors::AmdMonError;
use amdmond_lib::AmdMon;

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
    type Err = AmdMonError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "short" | "s" => Ok(MonitorFormat::Short),
            "verbose" | "v" | "long" | "l" => Ok(MonitorFormat::Verbose),
            _ => Err(AmdMonError::InvalidMonitorFormat),
        }
    }
}

#[derive(Debug, gumdrop::Options)]
pub struct Watch {
    #[options(help = "Help message")]
    help: bool,
    #[options(help = "Monitor format")]
    format: MonitorFormat,
}

impl Default for Watch {
    fn default() -> Self {
        Self {
            help: false,
            format: MonitorFormat::Short,
        }
    }
}

/// Start print cards temperature and fan speed
pub fn run(monitor: Watch, _config: monitor::Config) -> amdmond_lib::Result<()> {
    let fan_config = fan::load_config(DEFAULT_FAN_CONFIG_PATH)?;
    match monitor.format {
        MonitorFormat::Short => short(fan_config),
        MonitorFormat::Verbose => verbose(fan_config),
    }
}

pub fn verbose(config: fan::Config) -> amdmond_lib::Result<()> {
    let mut hw_mons = AmdMon::wrap_all(hw_mons(true)?, &config);

    loop {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        for hw_mon in hw_mons.iter_mut() {
            println!("Card {:3}", hw_mon.card().to_string().replace("card", ""));
            println!("  MIN |  MAX |  PWM   |   %");
            let min = hw_mon.pwm_min();
            let max = hw_mon.pwm_max();
            println!(
                " {:>4} | {:>4} | {:>6} | {:>3}",
                min,
                max,
                hw_mon
                    .pwm()
                    .map_or_else(|_e| String::from("FAILED"), |f| f.to_string()),
                (linear_map(
                    hw_mon.pwm().unwrap_or_default() as f64,
                    min as f64,
                    max as f64,
                    0f64,
                    100f64,
                ))
                .round(),
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

pub fn short(config: fan::Config) -> amdmond_lib::Result<()> {
    let mut hw_mons = AmdMon::wrap_all(hw_mons(true)?, &config);
    loop {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        for hw_mon in hw_mons.iter_mut() {
            println!(
                "Card {:3} | Temp     |  MIN |  MAX |  PWM |   %",
                hw_mon.card().to_string().replace("card", "")
            );
            let min = hw_mon.pwm_min();
            let max = hw_mon.pwm_max();
            println!(
                "         | {:>5.2}    | {:>4} | {:>4} | {:>4} | {:>3}",
                hw_mon.max_gpu_temp().unwrap_or_default(),
                min,
                max,
                hw_mon.pwm().unwrap_or_default(),
                linear_map(
                    hw_mon.pwm().unwrap_or_default() as f64,
                    min as f64,
                    max as f64,
                    0f64,
                    100f64,
                )
                .round(),
            );
        }
        std::thread::sleep(std::time::Duration::from_secs(4));
    }
}
