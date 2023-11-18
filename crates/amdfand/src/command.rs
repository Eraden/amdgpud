use amdgpu::hw_mon::HwMon;
use amdgpu::utils::{linear_map, load_temp_inputs};
use amdgpu::{
    utils, TempInput, PULSE_WIDTH_MODULATION_MANUAL, PULSE_WIDTH_MODULATION_MAX,
    PULSE_WIDTH_MODULATION_MIN, PULSE_WIDTH_MODULATION_MODE,
};
use amdgpu_config::fan::Config;
use gumdrop::Options;

use crate::{change_mode, service};

#[derive(Debug, Options)]
pub struct AvailableCards {
    #[options(help = "Help message")]
    help: bool,
}

#[derive(Debug, Options)]
pub enum FanCommand {
    #[options(help = "Check AMD GPU temperature and change fan speed depends on configuration")]
    Service(service::Service),
    #[options(help = "Switch GPU to automatic fan speed control")]
    SetAutomatic(change_mode::Switcher),
    #[options(help = "Switch GPU to manual fan speed control")]
    SetManual(change_mode::Switcher),
    #[options(help = "Print available cards")]
    Available(AvailableCards),
}

#[derive(Debug, onlyerror::Error)]
pub enum FanError {
    #[error("AMD GPU fan speed is malformed. It should be number. {0:?}")]
    NonIntPwm(std::num::ParseIntError),
    #[error("AMD GPU temperature is malformed. It should be number. {0:?}")]
    NonIntTemp(std::num::ParseIntError),
    #[error("Failed to read AMD GPU temperatures from tempX_input. No input was found")]
    EmptyTempSet,
    #[error("Unable to change fan speed to manual mode. {0}")]
    ManualSpeedFailed(utils::AmdGpuError),
    #[error("Unable to change fan speed to automatic mode. {0}")]
    AutomaticSpeedFailed(utils::AmdGpuError),
    #[error("Unable to change AMD GPU modulation (a.k.a. speed) to {value}. {error}")]
    FailedToChangeSpeed {
        value: u64,
        error: utils::AmdGpuError,
    },
}

pub struct Fan<Root: amdgpu::hw_mon::RootPath> {
    pub hw_mon: HwMon<Root>,
    /// List of available temperature inputs for current HW MOD
    pub temp_inputs: Vec<String>,
    /// Preferred temperature input
    pub temp_input: Option<TempInput>,
    /// Minimal modulation (between 0-255)
    pub pwm_min: Option<u32>,
    /// Maximal modulation (between 0-255)
    pub pwm_max: Option<u32>,
}

impl<Root: amdgpu::hw_mon::RootPath> std::ops::Deref for Fan<Root> {
    type Target = HwMon<Root>;

    fn deref(&self) -> &Self::Target {
        &self.hw_mon
    }
}

impl<Root: amdgpu::hw_mon::RootPath> std::ops::DerefMut for Fan<Root> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.hw_mon
    }
}

const MODULATION_ENABLED_FILE: &str = "pwm1_enable";

impl<Root: amdgpu::hw_mon::RootPath> Fan<Root> {
    pub fn wrap(hw_mon: HwMon<Root>, config: &Config) -> Self {
        Self {
            temp_input: config.temp_input().copied(),
            temp_inputs: load_temp_inputs(&hw_mon),
            hw_mon,
            pwm_min: None,
            pwm_max: None,
        }
    }

    pub fn wrap_all(v: Vec<HwMon<Root>>, config: &Config) -> Vec<Fan<Root>> {
        v.into_iter().map(|hw| Self::wrap(hw, config)).collect()
    }

    /// Change fan speed to given value if it's between minimal and maximal
    /// value
    pub fn set_speed(&mut self, speed: f64) -> crate::Result<()> {
        let min = self.pwm_min() as f64;
        let max = self.pwm_max() as f64;
        let pwm = linear_map(speed, 0f64, 100f64, min, max).round() as u64;
        self.write_pwm(pwm)?;
        Ok(())
    }

    /// Change gpu fan speed management to manual (amdfand will manage speed)
    /// instead of GPU embedded manager
    pub fn write_manual(&self) -> crate::Result<()> {
        self.hw_mon_write(MODULATION_ENABLED_FILE, 1)
            .map_err(FanError::ManualSpeedFailed)?;
        Ok(())
    }

    /// Change gpu fan speed management to automatic, speed will be managed by
    /// GPU embedded manager
    pub fn write_automatic(&self) -> crate::Result<()> {
        self.hw_mon_write("pwm1_enable", 2)
            .map_err(FanError::AutomaticSpeedFailed)?;
        Ok(())
    }

    /// Change fan speed to given value with checking min-max range
    fn write_pwm(&self, value: u64) -> crate::Result<()> {
        if !self.is_fan_manual() {
            self.write_manual()?;
        }
        self.hw_mon_write("pwm1", value)
            .map_err(|error| FanError::FailedToChangeSpeed { value, error })?;
        Ok(())
    }

    /// Check if gpu fan is managed by GPU embedded manager
    pub fn is_fan_manual(&self) -> bool {
        self.hw_mon_read(PULSE_WIDTH_MODULATION_MODE)
            .map(|s| s.as_str() == PULSE_WIDTH_MODULATION_MANUAL)
            .unwrap_or_default()
    }

    /// Get maximal GPU temperature from all inputs.
    /// This is not recommended since GPU can heat differently in different
    /// parts and usually only temp1 should be taken for consideration.
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

    /// Read temperature from given input sensor
    pub fn read_gpu_temp(&self, name: &str) -> crate::Result<u64> {
        let value = self
            .hw_mon_read(name)?
            .parse::<u64>()
            .map_err(FanError::NonIntTemp)?;
        Ok(value)
    }

    /// Read minimal fan speed. Usually this is 0
    pub fn pwm_min(&mut self) -> u32 {
        if self.pwm_min.is_none() {
            self.pwm_min = Some(self.value_or(PULSE_WIDTH_MODULATION_MIN, 0));
        };
        self.pwm_min.unwrap_or(0)
    }

    /// Read maximal fan speed. Usually this is 255
    pub fn pwm_max(&mut self) -> u32 {
        if self.pwm_max.is_none() {
            self.pwm_max = Some(self.value_or(PULSE_WIDTH_MODULATION_MAX, 255));
        };
        self.pwm_max.unwrap_or(255)
    }
}
