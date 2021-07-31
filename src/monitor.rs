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
