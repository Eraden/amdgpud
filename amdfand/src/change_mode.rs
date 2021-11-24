use crate::command::Fan;
use amdgpu::utils::hw_mons;
use gumdrop::Options;

use crate::config::Config;
use crate::{AmdFanError, FanMode};

/// Change card fan mode to either automatic or manual
pub fn run(switcher: Switcher, mode: FanMode, config: Config) -> crate::Result<()> {
    let mut hw_mons = Fan::wrap_all(hw_mons(true)?, &config);

    let cards = match switcher.card {
        Some(card_id) => match hw_mons.iter().position(|hw_mon| **hw_mon.card() == card_id) {
            Some(card) => vec![hw_mons.remove(card)],
            None => {
                eprintln!("Card does not exists. Available cards: ");
                for hw_mon in hw_mons {
                    eprintln!(" * {}", *hw_mon.card());
                }
                return Err(AmdFanError::NoAmdCardFound);
            }
        },
        None => hw_mons,
    };

    for hw_mon in cards {
        match mode {
            FanMode::Automatic => {
                if let Err(e) = hw_mon.write_automatic() {
                    log::error!("{:?}", e);
                }
            }
            FanMode::Manual => {
                if let Err(e) = hw_mon.write_manual() {
                    log::error!("{:?}", e);
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Options)]
pub struct Switcher {
    #[options(help = "Print help message")]
    help: bool,
    #[options(help = "GPU Card number")]
    card: Option<u32>,
}
