use std::io::ErrorKind;

use crate::hw_mon::HwMon;
use crate::{hw_mon, Card, CONFIG_DIR, ROOT_DIR};

/// linear mapping from the xrange to the yrange
pub fn linear_map(x: f64, x1: f64, x2: f64, y1: f64, y2: f64) -> f64 {
    let m = (y2 - y1) / (x2 - x1);
    m * (x - x1) + y1
}

/// Read all available graphic cards from direct rendering manager
pub fn read_cards() -> std::io::Result<Vec<Card>> {
    Ok(std::fs::read_dir(ROOT_DIR)?
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().as_os_str().to_str().map(String::from))
        .filter_map(|file_name| file_name.parse::<Card>().ok())
        .collect())
}

/// Wrap cards in HW Mon manipulator and
/// filter cards so only amd and listed in config cards are accessible
pub fn hw_mons(filter: bool) -> std::io::Result<Vec<HwMon>> {
    Ok(read_cards()?
        .into_iter()
        .map(|card| {
            log::info!("opening hw mon for {:?}", card);
            hw_mon::open_hw_mon(card)
        })
        .flatten()
        .filter(|hw_mon| {
            !filter || {
                log::info!("is vendor ok? {}", hw_mon.is_amd());
                hw_mon.is_amd()
            }
        })
        .filter(|hw_mon| {
            !filter || {
                log::info!("is hwmon name ok? {}", hw_mon.name_is_amd());
                hw_mon.name_is_amd()
            }
        })
        .collect())
}

pub fn ensure_config<Config, Error, P>(config_path: P) -> std::result::Result<Config, Error>
where
    Config: serde::Serialize + serde::de::DeserializeOwned + Default + Sized,
    P: AsRef<std::path::Path>,
    Error: From<std::io::Error>,
{
    match std::fs::read_to_string(&config_path) {
        Ok(s) => Ok(toml::from_str::<Config>(s.as_str()).unwrap()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let config = Config::default();
            std::fs::write(config_path, toml::to_string(&config).unwrap())?;
            Ok(config)
        }
        Err(e) => {
            log::error!("{:?}", e);
            panic!();
        }
    }
}

pub fn load_temp_inputs(hw_mon: &HwMon) -> Vec<String> {
    let dir = match std::fs::read_dir(hw_mon.mon_dir()) {
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

pub fn ensure_config_dir() -> std::io::Result<()> {
    if std::fs::read(CONFIG_DIR).map_err(|e| e.kind() == ErrorKind::NotFound) == Err(true) {
        std::fs::create_dir_all(CONFIG_DIR)?;
    }
    Ok(())
}
