use crate::config::Card;
use crate::io_err::{invalid_input, not_found};
use crate::utils::linear_map;
use crate::{AmdFanError, HwMon, HW_MON_DIR, ROOT_DIR};

/// pulse width modulation fan control minimum level (0)
const PULSE_WIDTH_MODULATION_MIN: &str = "pwm1_min";

/// pulse width modulation fan control maximum level (255)
const PULSE_WIDTH_MODULATION_MAX: &str = "pwm1_max";

/// pulse width modulation fan level (0-255)
const PULSE_WIDTH_MODULATION: &str = "pwm1";

/// pulse width modulation fan control method (0: no fan speed control, 1: manual fan speed control using pwm interface, 2: automatic fan speed control)
const PULSE_WIDTH_MODULATION_MODE: &str = "pwm1_enable";

// static PULSE_WIDTH_MODULATION_DISABLED: &str = "0";
const PULSE_WIDTH_MODULATION_AUTO: &str = "2";
const PULSE_WIDTH_MODULATION_MANUAL: &str = "1";

impl HwMon {
    pub fn new(card: &Card, name: &str) -> Self {
        Self {
            card: *card,
            name: String::from(name),
            pwm_min: None,
            pwm_max: None,
            temp_inputs: load_temp_inputs(card, name),
        }
    }

    pub fn max_gpu_temp(&self) -> std::io::Result<f64> {
        let mut results = Vec::with_capacity(self.temp_inputs.len());
        for name in self.temp_inputs.iter() {
            results.push(self.read_gpu_temp(name).unwrap_or(0));
        }
        results.sort_unstable();
        results
            .last()
            .copied()
            .map(|temp| temp as f64 / 1000f64)
            .ok_or_else(invalid_input)
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
            invalid_input()
        })
    }

    #[inline]
    pub(crate) fn name(&self) -> std::io::Result<String> {
        self.read("name")
    }

    pub fn pwm_min(&mut self) -> u32 {
        if self.pwm_min.is_none() {
            self.pwm_min = Some(self.value_or(PULSE_WIDTH_MODULATION_MIN, 0));
        };
        self.pwm_min.unwrap_or_default()
    }

    pub fn pwm_max(&mut self) -> u32 {
        if self.pwm_max.is_none() {
            self.pwm_max = Some(self.value_or(PULSE_WIDTH_MODULATION_MAX, 255));
        };
        self.pwm_max.unwrap_or(255)
    }

    pub fn pwm(&self) -> std::io::Result<u32> {
        self.read(PULSE_WIDTH_MODULATION)?.parse().map_err(|_e| {
            log::warn!("Read from gpu monitor failed. Invalid pwm value");
            invalid_input()
        })
    }

    pub fn is_fan_manual(&self) -> bool {
        self.read(PULSE_WIDTH_MODULATION_MODE)
            .map(|s| s.as_str() == PULSE_WIDTH_MODULATION_MANUAL)
            .unwrap_or_default()
    }

    pub fn is_fan_automatic(&self) -> bool {
        self.read(PULSE_WIDTH_MODULATION_MODE)
            .map(|s| s.as_str() == PULSE_WIDTH_MODULATION_AUTO)
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

    pub fn set_pwm(&self, value: u64) -> std::io::Result<()> {
        if self.is_fan_automatic() {
            self.set_manual()?;
        }
        self.write("pwm1", value)
    }

    pub fn set_speed(&mut self, speed: f64) -> std::io::Result<()> {
        let min = self.pwm_min() as f64;
        let max = self.pwm_max() as f64;
        let pwm = linear_map(speed, 0f64, 100f64, min, max).round() as u64;
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
    fn value_or<R: std::str::FromStr>(&self, name: &str, fallback: R) -> R {
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
    hw_mon_dirs_path(card).join(name)
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
        .ok_or_else(not_found)?;
    Ok(HwMon::new(&card, &name))
}
