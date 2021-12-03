use amdgpu::hw_mon::HwMon;

use crate::{Config, VoltageError};
use crate::apply_changes::ApplyChanges;
use crate::change_state::ChangeState;
use crate::clock_state::{ClockState, Frequency, Voltage};
use crate::print_states::PrintStates;
use crate::setup_info::SetupInfo;

#[derive(Debug)]
pub enum HardwareModule {
    Engine,
    Memory,
}

impl std::str::FromStr for HardwareModule {
    type Err = VoltageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "memory" => Ok(HardwareModule::Memory),
            "engine" => Ok(HardwareModule::Engine),
            _ => Err(VoltageError::UnknownHardwareModule(s.to_string())),
        }
    }
}

#[derive(Debug, gumdrop::Options)]
pub enum VoltageCommand {
    SetupInfo(SetupInfo),
    PrintStates(PrintStates),
    ChangeState(ChangeState),
    ApplyChanges(ApplyChanges),
}

pub struct VoltageManipulator {
    hw_mon: HwMon,
}

impl std::ops::Deref for VoltageManipulator {
    type Target = HwMon;

    fn deref(&self) -> &Self::Target {
        &self.hw_mon
    }
}

impl std::ops::DerefMut for VoltageManipulator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.hw_mon
    }
}

impl VoltageManipulator {
    pub fn wrap(hw_mon: HwMon, _config: &Config) -> Self {
        Self { hw_mon }
    }

    pub fn wrap_all(mons: Vec<HwMon>, config: &Config) -> Vec<Self> {
        mons.into_iter()
            .map(|mon| Self::wrap(mon, config))
            .collect()
    }

    pub fn write_apply(&self) -> crate::Result<()> {
        self.device_write("pp_od_clk_voltage", "c")?;
        Ok(())
    }

    pub fn write_state(
        &self,
        state_index: u16,
        freq: Frequency,
        voltage: Voltage,
        module: HardwareModule,
    ) -> crate::Result<()> {
        self.device_write(
            "pp_od_clk_voltage",
            format!(
                "{module} {state_index} {freq} {voltage}",
                state_index = state_index,
                freq = freq.to_string(),
                voltage = voltage.to_string(),
                module = match module {
                    HardwareModule::Engine => "s",
                    HardwareModule::Memory => "m",
                },
            ),
        )?;
        Ok(())
    }

    pub fn clock_states(&self) -> crate::Result<ClockState> {
        let state = self.device_read("pp_od_clk_voltage")?.parse()?;
        Ok(state)
    }
}
