use crate::command::Fan;
use crate::AmdFanError;
use amdgpu::utils::hw_mons;
use gumdrop::Options;

use crate::config::Config;

/// Start service which will change fan speed according to config and GPU temperature
pub fn run(config: Config) -> crate::Result<()> {
    let mut hw_mons = Fan::wrap_all(hw_mons(true)?, &config);

    if hw_mons.is_empty() {
        return Err(AmdFanError::NoHwMonFound);
    }
    let mut cache = std::collections::HashMap::new();
    loop {
        for hw_mon in hw_mons.iter_mut() {
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

            log::debug!("Current {} temperature: {}", hw_mon.card(), gpu_temp);
            let last = *cache.entry(**hw_mon.card()).or_insert(1_000f64);

            if (last - gpu_temp).abs() < 0.001f64 {
                log::debug!("Temperature didn't change");
                continue;
            };
            let speed = config.speed_for_temp(gpu_temp);
            log::debug!("Resolved speed {:.2}", speed);

            if let Err(e) = hw_mon.set_speed(speed) {
                log::error!("Failed to change speed to {}. {:?}", speed, e);
            }
            cache.insert(**hw_mon.card(), gpu_temp);
        }
        std::thread::sleep(std::time::Duration::from_secs(4));
    }
}

#[derive(Debug, Options)]
pub struct Service {
    #[options(help = "Help message")]
    help: bool,
}
