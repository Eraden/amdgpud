use gumdrop::Options;

pub mod change_mode;
pub mod monitor;
pub mod service;

#[derive(Debug, Options)]
pub struct AvailableCards {
    #[options(help = "Help message")]
    help: bool,
}

#[derive(Debug, Options)]
pub enum FanCommand {
    #[options(help = "Print current temp and fan speed")]
    Monitor(monitor::Monitor),
    #[options(help = "Set fan speed depends on GPU temperature")]
    Service(service::Service),
    #[options(help = "Switch to GPU automatic fan speed control")]
    SetAutomatic(change_mode::Switcher),
    #[options(help = "Switch to GPU manual fan speed control")]
    SetManual(change_mode::Switcher),
    #[options(help = "Print available cards")]
    Available(AvailableCards),
}
