use crate::{change_mode, monitor, service, Config};
use amdgpu::hw_mon::HwMon;
use amdgpu::utils::linear_map;
use amdgpu::TempInput;
use gumdrop::Options;

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

#[derive(Debug, Options)]
pub struct AvailableCards {
    #[options(help = "Help message")]
    help: bool,
}

#[derive(Debug, Options)]
pub enum FanCommand {
    #[options(help = "Print current temperature and fan speed")]
    Monitor(monitor::Monitor),
    #[options(help = "Check AMD GPU temperature and change fan speed depends on configuration")]
    Service(service::Service),
    #[options(help = "Switch GPU to automatic fan speed control")]
    SetAutomatic(change_mode::Switcher),
    #[options(help = "Switch GPU to manual fan speed control")]
    SetManual(change_mode::Switcher),
    #[options(help = "Print available cards")]
    Available(AvailableCards),
}

#[derive(Debug, thiserror::Error)]
pub enum FanError {
    #[error("AMD GPU fan speed is malformed. It should be number. {0:?}")]
    NonIntPwm(std::num::ParseIntError),
    #[error("AMD GPU temperature is malformed. It should be number. {0:?}")]
    NonIntTemp(std::num::ParseIntError),
    #[error("Failed to read AMD GPU temperatures from tempX_input. No input was found")]
    EmptyTempSet,
    #[error("Unable to change fan speed to manual mode. {0}")]
    ManualSpeedFailed(std::io::Error),
    #[error("Unable to change fan speed to automatic mode. {0}")]
    AutomaticSpeedFailed(std::io::Error),
    #[error("Unable to change AMD GPU modulation (a.k.a. speed) to {value}. {error}")]
    FailedToChangeSpeed { value: u64, error: std::io::Error },
}

pub struct Fan {
    pub hw_mon: HwMon,
    /// List of available temperature inputs for current HW MOD
    pub temp_inputs: Vec<String>,
    /// Preferred temperature input
    pub temp_input: Option<TempInput>,
    /// Minimal modulation (between 0-255)
    pub pwm_min: Option<u32>,
    /// Maximal modulation (between 0-255)
    pub pwm_max: Option<u32>,
}

impl std::ops::Deref for Fan {
    type Target = HwMon;

    fn deref(&self) -> &Self::Target {
        &self.hw_mon
    }
}

impl std::ops::DerefMut for Fan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.hw_mon
    }
}

impl Fan {
    pub fn wrap(hw_mon: HwMon, config: &Config) -> Self {
        Self {
            temp_input: config.temp_input().copied(),
            temp_inputs: load_temp_inputs(&hw_mon),
            hw_mon,
            pwm_min: None,
            pwm_max: None,
        }
    }

    pub fn wrap_all(v: Vec<HwMon>, config: &Config) -> Vec<Fan> {
        v.into_iter().map(|hw| Self::wrap(hw, config)).collect()
    }

    pub(crate) fn set_speed(&mut self, speed: f64) -> crate::Result<()> {
        let min = self.pwm_min() as f64;
        let max = self.pwm_max() as f64;
        let pwm = linear_map(speed, 0f64, 100f64, min, max).round() as u64;
        self.write_pwm(pwm)?;
        Ok(())
    }

    pub(crate) fn write_manual(&self) -> crate::Result<()> {
        self.hw_mon_write("pwm1_enable", 1)
            .map_err(FanError::ManualSpeedFailed)?;
        Ok(())
    }

    pub(crate) fn write_automatic(&self) -> crate::Result<()> {
        self.hw_mon_write("pwm1_enable", 2)
            .map_err(FanError::AutomaticSpeedFailed)?;
        Ok(())
    }

    fn write_pwm(&self, value: u64) -> crate::Result<()> {
        if self.is_fan_automatic() {
            self.write_manual()?;
        }
        self.hw_mon_write("pwm1", value)
            .map_err(|error| FanError::FailedToChangeSpeed { value, error })?;
        Ok(())
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

    pub fn pwm(&self) -> crate::Result<u32> {
        let value = self
            .hw_mon_read(PULSE_WIDTH_MODULATION)?
            .parse()
            .map_err(FanError::NonIntPwm)?;
        Ok(value)
    }

    pub fn is_fan_automatic(&self) -> bool {
        self.hw_mon_read(PULSE_WIDTH_MODULATION_MODE)
            .map(|s| s.as_str() == PULSE_WIDTH_MODULATION_AUTO)
            .unwrap_or_default()
    }

    pub fn max_gpu_temp(&self) -> crate::Result<f64> {
        if let Some(input) = self.temp_input.as_ref() {
            let value = self.read_gpu_temp(&input.as_string())?;
            return Ok(value as f64 / 1000f64);
        }
        let mut results = Vec::with_capacity(self.temp_inputs.len());
        for name in self.temp_inputs.iter() {
            results.push(self.read_gpu_temp(name).unwrap_or(0));
        }
        results.sort_unstable();
        let value = results
            .last()
            .copied()
            .map(|temp| temp as f64 / 1000f64)
            .ok_or(FanError::EmptyTempSet)?;
        Ok(value)
    }

    pub fn gpu_temp(&self) -> Vec<(String, crate::Result<f64>)> {
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

    pub(crate) fn read_gpu_temp(&self, name: &str) -> crate::Result<u64> {
        let value = self
            .hw_mon_read(name)?
            .parse::<u64>()
            .map_err(FanError::NonIntTemp)?;
        Ok(value)
    }
}

fn load_temp_inputs(hw_mon: &HwMon) -> Vec<String> {
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
