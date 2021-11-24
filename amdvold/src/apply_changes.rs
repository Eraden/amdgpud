use crate::command::VoltageManipulator;
use crate::{Config, VoltageError};
use amdgpu::utils::hw_mons;

#[derive(Debug, gumdrop::Options)]
pub struct ApplyChanges {
    help: bool,
}

pub fn run(_command: ApplyChanges, config: &Config) -> crate::Result<()> {
    let mut mons = VoltageManipulator::wrap_all(hw_mons(false)?, config);
    if mons.is_empty() {
        return Err(VoltageError::NoAmdGpu);
    }
    let mon = mons.remove(0);
    mon.write_apply()?;
    Ok(())
}
