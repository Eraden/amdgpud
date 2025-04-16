use std::fmt::Formatter;

use amdgpu::utils::hw_mons;
use amdgpu::{config_reloaded, is_reload_required, listen_unix_signal};
use amdgpu_config::fan::Config;
use gumdrop::Options;
use tracing::{debug, error, info};

use crate::command::Fan;
use crate::AmdFanError;

/// Start service which will change fan speed according to config and GPU
/// temperature
pub fn run(mut config: Config) -> crate::Result<()> {
    listen_unix_signal();

    let mut hw_mons = Fan::wrap_all(hw_mons(true)?, &config);

    if hw_mons.is_empty() {
        return Err(AmdFanError::NoHwMonFound);
    }
    hw_mons.iter().for_each(|fan| {
        if let Err(e) = fan.write_manual() {
            debug!(
                "Failed to switch to manual fan manipulation for fan {:?}. {:?}",
                fan.hw_mon, e
            );
        }
    });

    let mut cache = std::collections::HashMap::new();
    loop {
        if is_reload_required() {
            info!("Reloading config...");
            config = config.reload()?;
            info!("  config reloaded");
            config_reloaded();
        }
        hw_mons
            .iter_mut()
            .map(|hw_mon| {
                (
                    hw_mon,
                    highest_speed(&config, hw_mon),
                    *cache.entry(**hw_mon.card()).or_insert(-1_f64),
                )
            })
            .filter(|_, last, speed| (last - speed).abs() < 0.001f64)
            .for_each(|(hw_mon, speed, last)| {
                debug!("Changing speed to {speed:0.2}");

                if let Err(e) = hw_mon.set_speed(speed) {
                    error!("Failed to change speed to {}. {:?}", speed, e);
                }
                cache.insert(**hw_mon.card(), speed);
            });
        std::thread::sleep(std::time::Duration::from_millis(config.update_rate()));
    }
}

fn highest_speed<Root: amdgpu::hw_mon::RootPath>(config: &Config, hw_mon: &Fan<Root>) -> f64 {
    let gpu_temp = config
        .temp_input()
        .and_then(|input| {
            hw_mon
                .read_gpu_temp(&input.as_string())
                .map(|temp| temp as f64 / 1000f64)
                .ok()
        })
        .or_else(|| hw_mon.max_gpu_temp().ok())
        .unwrap_or_default();

    let gpu_usage = hw_mon.gpu_usage().unwrap_or(0.0);

    let temp_fan_speed = config.fan_speed_for_temp(gpu_temp);
    let usage_fan_speed = config.fan_speed_for_usage(gpu_usage);
    let value = temp_fan_speed.max(usage_fan_speed);

    debug!(
        "{:?}",
        Diagnostic {
            gpu_temp,
            gpu_usage,
            temp_fan_speed,
            usage_fan_speed,
            value,
        }
    );

    value
}

#[allow(dead_code)]
struct Diagnostic {
    gpu_temp: f64,
    gpu_usage: f64,
    temp_fan_speed: f64,
    usage_fan_speed: f64,
    value: f64,
}

impl std::fmt::Debug for Diagnostic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ gpu_temp: {:0.2}, gpu_usage: {:0.2}, temp_fan_speed: {:0.2}, usage_fan_speed: {:0.2}, value: {:0.2} }}",
            self.gpu_temp, self.gpu_usage, self.temp_fan_speed, self.usage_fan_speed, self.value
        )
    }
}

#[derive(Debug, Options)]
pub struct Service {
    #[options(help = "Help message")]
    help: bool,
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use amdgpu::hw_mon::{HwMon, HwMonName, RootPath};
    use amdgpu::Card;
    use amdgpu_config::fan::Config;

    use crate::command::Fan;
    use crate::service::highest_speed;

    struct FakeSysFs<'root> {
        root: &'root Path,
    }

    impl<'root> RootPath for FakeSysFs<'root> {
        fn root_dir(&self) -> PathBuf {
            self.root.to_path_buf()
        }

        fn device_dir(&self, card: &Card) -> PathBuf {
            self.root_dir().join(card.to_string())
        }

        fn mon_dir(&self, card: &Card, name: &HwMonName) -> PathBuf {
            self.device_dir(card).join(name.as_str())
        }
    }

    #[test]
    fn precent_is_higher() {
        let dir = tempdir::TempDir::new("foo").unwrap();
        let fs = FakeSysFs { root: dir.path() };

        let card = Card(0);
        let name = HwMonName("a".into());
        std::fs::create_dir_all(fs.mon_dir(&card, &name)).unwrap();

        std::fs::write(fs.device_dir(&card).join("gpu_busy_percent"), "33").unwrap();
        std::fs::write(fs.mon_dir(&card, &name).join("temp1_input"), "12").unwrap();

        let config = Config::default();
        let hw = HwMon::<FakeSysFs>::new(&card, name, fs);

        let usage = hw.read_gpu_usage();
        assert_eq!(usage.unwrap(), 33);

        let fan = Fan::wrap(hw, &config);

        let value = highest_speed(&config, &fan);
        assert_eq!(value, 33.0);
    }

    #[test]
    fn temp_is_higher() {
        let dir = tempdir::TempDir::new("foo").unwrap();
        let fs = FakeSysFs { root: dir.path() };

        let card = Card(0);
        let name = HwMonName("a".into());
        std::fs::create_dir_all(fs.mon_dir(&card, &name)).unwrap();

        std::fs::write(fs.device_dir(&card).join("gpu_busy_percent"), "33").unwrap();
        std::fs::write(fs.mon_dir(&card, &name).join("temp1_input"), "120").unwrap();

        let config = Config::default();
        let hw = HwMon::<FakeSysFs>::new(&card, name, fs);

        let usage = hw.read_gpu_usage();
        assert_eq!(usage.unwrap(), 33);

        let fan = Fan::wrap(hw, &config);

        let value = highest_speed(&config, &fan);
        assert_eq!(value, 33.0);
    }
}
