pub mod errors;

use crate::errors::AmdMonError;
use amdgpu::hw_mon::HwMon;
use amdgpu::utils::load_temp_inputs;
use amdgpu::{
    TempInput, PULSE_WIDTH_MODULATION, PULSE_WIDTH_MODULATION_MAX, PULSE_WIDTH_MODULATION_MIN,
};
use amdgpu_config::fan;

pub type Result<T> = std::result::Result<T, AmdMonError>;

pub struct AmdMon {
    temp_input: Option<TempInput>,
    inputs: Vec<String>,
    hw_mon: HwMon,
    /// Minimal modulation (between 0-255)
    pub pwm_min: Option<u32>,
    /// Maximal modulation (between 0-255)
    pub pwm_max: Option<u32>,
}

impl std::ops::Deref for AmdMon {
    type Target = HwMon;

    fn deref(&self) -> &Self::Target {
        &self.hw_mon
    }
}

impl AmdMon {
    pub fn wrap_all(mons: Vec<HwMon>, config: &fan::Config) -> Vec<Self> {
        mons.into_iter()
            .map(|hw_mon| Self::wrap(hw_mon, config))
            .collect()
    }

    pub fn wrap(hw_mon: HwMon, config: &fan::Config) -> Self {
        Self {
            temp_input: config.temp_input().cloned(),
            inputs: load_temp_inputs(&hw_mon),
            hw_mon,
            pwm_min: None,
            pwm_max: None,
        }
    }

    pub fn gpu_temp(&self) -> Vec<(String, crate::Result<f64>)> {
        self.inputs
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

    pub fn gpu_temp_of(&self, input_idx: usize) -> Option<(&String, crate::Result<f64>)> {
        self.inputs.get(input_idx).map(|name| {
            let temp = self
                .read_gpu_temp(name.as_str())
                .map(|temp| temp as f64 / 1000f64);
            (name, temp)
        })
    }

    pub fn read_gpu_temp(&self, name: &str) -> crate::Result<u64> {
        let value = self
            .hw_mon_read(name)?
            .parse::<u64>()
            .map_err(AmdMonError::NonIntTemp)?;
        Ok(value)
    }

    pub fn pwm(&self) -> crate::Result<u32> {
        let value = self
            .hw_mon_read(PULSE_WIDTH_MODULATION)?
            .parse()
            .map_err(AmdMonError::NonIntPwm)?;
        Ok(value)
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

    pub fn max_gpu_temp(&self) -> crate::Result<f64> {
        if let Some(input) = self.temp_input.as_ref() {
            let value = self.read_gpu_temp(&input.as_string())?;
            return Ok(value as f64 / 1000f64);
        }
        let mut results = Vec::with_capacity(self.inputs.len());
        for name in self.inputs.iter() {
            results.push(self.read_gpu_temp(name).unwrap_or(0));
        }
        results.sort_unstable();
        let value = results
            .last()
            .copied()
            .map(|temp| temp as f64 / 1000f64)
            .ok_or(AmdMonError::EmptyTempSet)?;
        Ok(value)
    }
}
