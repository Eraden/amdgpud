use amdgpu::utils::hw_mons;
use amdgpu_config::fan;
use amdgpu_config::fan::DEFAULT_FAN_CONFIG_PATH;
use amdgpu_config::monitor::Config;
use amdmond_lib::errors::AmdMonError;
use amdmond_lib::AmdMon;

#[derive(gumdrop::Options)]
pub struct LogFile {
    #[options(help = "Help message")]
    help: bool,
    #[options(help = "Full path to statistics file")]
    stat_file: String,
    #[options(help = "Time between each check. 1000 is 1s, by default 5s")]
    interval: Option<u32>,
}

#[derive(serde::Serialize)]
struct Stat {
    time: chrono::DateTime<chrono::Local>,
    temperature: f64,
    modulation: u32,
    speed_setting: f64,
    temperature_setting: f64,
}

pub fn run(command: LogFile, config: Config) -> amdmond_lib::Result<()> {
    let fan_config = fan::load_config(DEFAULT_FAN_CONFIG_PATH)?;

    let duration = std::time::Duration::new(
        0,
        command.interval.unwrap_or_else(|| config.interval()) * 1_000_000,
    );
    log::info!("Updating each: {:?}", duration);

    let _ = std::fs::remove_file(command.stat_file.as_str());
    let stat_file = std::fs::OpenOptions::new()
        .create(true)
        .append(false)
        .write(true)
        .open(command.stat_file.as_str())?;

    let mut writer = csv::WriterBuilder::new()
        .double_quote(true)
        .has_headers(true)
        .buffer_capacity(100)
        .from_writer(stat_file);

    let mon = {
        let mons = hw_mons(true)?;
        if mons.is_empty() {
            return Err(AmdMonError::NoHwMon);
        }
        AmdMon::wrap(hw_mons(true)?.remove(0), &fan_config)
    };

    loop {
        let time = chrono::Local::now();
        let temperature = {
            let mut temperatures = mon.gpu_temp();

            if let Some(input) = fan_config.temp_input() {
                let input_name = input.as_string();
                temperatures
                    .into_iter()
                    .find(|(name, _value)| name == &input_name)
                    .and_then(|(_, v)| v.ok())
                    .unwrap_or_default()
            } else if temperatures.len() > 1 {
                temperatures.remove(1).1.unwrap_or_default()
            } else {
                temperatures
                    .get(0)
                    .and_then(|(_, v)| v.as_ref().ok().copied())
                    .unwrap_or_default()
            }
        };
        let (speed_setting, temperature_setting) =
            if let Some(speed_matrix_point) = fan_config.speed_matrix_point(temperature) {
                (speed_matrix_point.speed, speed_matrix_point.temp)
            } else {
                Default::default()
            };

        let stat = Stat {
            time,
            temperature,
            modulation: mon.pwm()?,
            speed_setting,
            temperature_setting,
        };

        writer.serialize(stat)?;
        writer.flush()?;

        std::thread::sleep(duration);
    }
}
