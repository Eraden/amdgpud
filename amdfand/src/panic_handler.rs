use amdgpu::utils::hw_mons;

use crate::command::Fan;

pub fn restore_automatic() {
    for hw in hw_mons(true).unwrap_or_default() {
        if let Err(error) = (Fan {
            hw_mon: hw,
            temp_inputs: vec![],
            temp_input: None,
            pwm_min: None,
            pwm_max: None,
        })
            .write_automatic()
        {
            log::error!("{}", error);
        }
    }
}
