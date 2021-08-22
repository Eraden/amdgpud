use gumdrop::Options;

use crate::config::Config;
use crate::io_err::not_found;

/// Start service which will change fan speed according to config and GPU temperature
pub fn run(config: Config) -> std::io::Result<()> {
    let mut controllers = crate::utils::hw_mons(&config, true)?;
    if controllers.is_empty() {
        return Err(not_found());
    }
    let mut cache = std::collections::HashMap::new();
    loop {
        for hw_mon in controllers.iter_mut() {
            let gpu_temp = hw_mon.max_gpu_temp().unwrap_or_default();
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
