use crate::config::Config;

pub static ENABLE_VOLTAGE_INFO: &str = include_str!("../assets/enable_voltage_info.txt");

#[derive(Debug, gumdrop::Options)]
pub struct SetupInfo {
    help: bool,
}

pub fn run(_command: SetupInfo, _config: &Config) -> crate::Result<()> {
    println!("{}", ENABLE_VOLTAGE_INFO);
    Ok(())
}
