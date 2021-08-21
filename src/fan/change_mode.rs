use gumdrop::Options;

use crate::config::Config;
use crate::io_err::not_found;
use crate::FanMode;

/// Change card fan mode to either automatic or manual
pub fn run(switcher: Switcher, mode: FanMode, config: Config) -> std::io::Result<()> {
    let mut hw_mons = crate::utils::hw_mons(&config, true)?;

    let cards = match switcher.card {
        Some(card_id) => match hw_mons.iter().position(|hw_mon| **hw_mon.card() == card_id) {
            Some(card) => vec![hw_mons.remove(card)],
            None => {
                eprintln!("Card does not exists. Available cards: ");
                for hw_mon in hw_mons {
                    eprintln!(" * {}", *hw_mon.card());
                }
                return Err(not_found());
            }
        },
        None => hw_mons,
    };

    for hw_mon in cards {
        match mode {
            FanMode::Automatic => {
                if let Err(e) = hw_mon.set_automatic() {
                    log::error!("{:?}", e);
                }
            }
            FanMode::Manual => {
                if let Err(e) = hw_mon.set_manual() {
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
