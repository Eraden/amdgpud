use amdgpu::utils::hw_mons;

use crate::clock_state::{Frequency, Voltage};
use crate::command::{HardwareModule, VoltageManipulator};
use crate::{Config, VoltageError};

#[derive(Debug, thiserror::Error)]
pub enum ChangeStateError {
    #[error("No profile index was given")]
    Index,
    #[error("No frequency was given")]
    Freq,
    #[error("No voltage was given")]
    Voltage,
    #[error("No AMD GPU module was given (either memory or engine)")]
    Module,
}

#[derive(Debug, gumdrop::Options)]
pub struct ChangeState {
    #[options(help = "Help message")]
    help: bool,
    #[options(help = "Profile number", free)]
    index: u16,
    #[options(help = "Either memory or engine", free)]
    module: Option<HardwareModule>,
    #[options(help = "New GPU module frequency", free)]
    frequency: Option<Frequency>,
    #[options(help = "New GPU module voltage", free)]
    voltage: Option<Voltage>,
    #[options(help = "Apply changes immediately after change")]
    apply_immediately: bool,
}

pub fn run(command: ChangeState, config: &Config) -> crate::Result<()> {
    let mut mons = VoltageManipulator::wrap_all(hw_mons(false)?, config);
    if mons.is_empty() {
        return Err(VoltageError::NoAmdGpu);
    }
    let mon = mons.remove(0);
    let ChangeState {
        help: _,
        index,
        module,
        frequency,
        voltage,
        apply_immediately,
    } = command;
    mon.write_state(
        index,
        frequency.ok_or(ChangeStateError::Freq)?,
        voltage.ok_or(ChangeStateError::Voltage)?,
        module.ok_or(ChangeStateError::Module)?,
    )?;
    if apply_immediately {
        mon.write_apply()?;
    }

    Ok(())
}
