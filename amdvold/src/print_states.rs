use crate::command::VoltageManipulator;
use crate::Config;
use amdgpu::utils::hw_mons;

#[derive(Debug, gumdrop::Options)]
pub struct PrintStates {
    help: bool,
}

pub fn run(_command: PrintStates, config: Config) -> crate::Result<()> {
    let mons = VoltageManipulator::wrap_all(hw_mons(false)?, &config);
    for mon in mons {
        let states = mon.clock_states()?;
        println!("Engine clock frequencies:");
        if let Some(freq) = states.engine_label_lowest {
            println!("  LOWEST {}", freq.to_string());
        }
        if let Some(freq) = states.engine_label_highest {
            println!("  HIGHEST {}", freq.to_string());
        }
        println!();
        println!("Memory clock frequencies:");
        if let Some(freq) = states.memory_label_lowest {
            println!("  LOWEST {}", freq.to_string());
        }
        if let Some(freq) = states.memory_label_highest {
            println!("  HIGHEST {}", freq.to_string());
        }
        println!();
        println!("Curves:");
        for curve in states.curve_labels.iter() {
            println!(
                "  {:>10} {:>10}",
                curve.freq.to_string(),
                curve.voltage.to_string()
            );
        }
        println!();
    }
    Ok(())
}
