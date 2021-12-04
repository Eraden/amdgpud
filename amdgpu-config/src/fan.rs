use amdgpu::utils::{ensure_config, linear_map};
use amdgpu::{LogLevel, TempInput};

pub static DEFAULT_FAN_CONFIG_PATH: &str = "/etc/amdfand/config.toml";

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, Default, PartialEq)]
pub struct MatrixPoint {
    pub temp: f64,
    pub speed: f64,
}

impl MatrixPoint {
    pub const MIN: MatrixPoint = MatrixPoint {
        temp: 0.0,
        speed: 0.0,
    };
    pub const MAX: MatrixPoint = MatrixPoint {
        temp: 100.0,
        speed: 100.0,
    };

    pub fn new(temp: f64, speed: f64) -> Self {
        Self { temp, speed }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Config {
    #[serde(skip)]
    path: String,
    /// One of temperature inputs /sys/class/drm/card{X}/device/hwmon/hwmon{Y}/temp{Z}_input
    /// If nothing is provided higher reading will be taken (this is not good!)
    temp_input: Option<TempInput>,
    log_level: LogLevel,
    cards: Option<Vec<String>>,
    speed_matrix: Vec<MatrixPoint>,
}

impl Config {
    #[deprecated(
        since = "1.0.6",
        note = "Multi-card used is halted until we will have PC with multiple AMD GPU"
    )]
    pub fn cards(&self) -> Option<&Vec<String>> {
        self.cards.as_ref()
    }

    pub fn reload(self) -> Result<Config, ConfigError> {
        let config = load_config(&self.path)?;
        Ok(config)
    }

    pub fn speed_matrix(&self) -> &[MatrixPoint] {
        &self.speed_matrix
    }

    pub fn speed_matrix_mut(&mut self) -> &mut [MatrixPoint] {
        &mut self.speed_matrix
    }

    pub fn speed_matrix_vec_mut(&mut self) -> &mut Vec<MatrixPoint> {
        &mut self.speed_matrix
    }

    pub fn speed_matrix_point(&self, temp: f64) -> Option<&MatrixPoint> {
        match self.speed_matrix.iter().rposition(|p| p.temp <= temp) {
            Some(idx) => self.speed_matrix.get(idx),
            _ => None,
        }
    }

    pub fn speed_for_temp(&self, temp: f64) -> f64 {
        let idx = match self.speed_matrix.iter().rposition(|p| p.temp <= temp) {
            Some(idx) => idx,
            _ => return self.min_speed(),
        };

        if idx == self.speed_matrix.len() - 1 {
            return self.max_speed();
        }

        linear_map(
            temp,
            self.speed_matrix[idx].temp,
            self.speed_matrix[idx + 1].temp,
            self.speed_matrix[idx].speed,
            self.speed_matrix[idx + 1].speed,
        )
    }

    pub fn log_level(&self) -> LogLevel {
        self.log_level
    }

    pub fn temp_input(&self) -> Option<&TempInput> {
        self.temp_input.as_ref()
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    fn min_speed(&self) -> f64 {
        self.speed_matrix.first().map(|p| p.speed).unwrap_or(0f64)
    }

    fn max_speed(&self) -> f64 {
        self.speed_matrix.last().map(|p| p.speed).unwrap_or(100f64)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: String::from(DEFAULT_FAN_CONFIG_PATH),
            #[allow(deprecated)]
            cards: None,
            log_level: LogLevel::Error,
            speed_matrix: vec![
                MatrixPoint {
                    temp: 4f64,
                    speed: 4f64,
                },
                MatrixPoint {
                    temp: 30f64,
                    speed: 33f64,
                },
                MatrixPoint {
                    temp: 45f64,
                    speed: 50f64,
                },
                MatrixPoint {
                    temp: 60f64,
                    speed: 66f64,
                },
                MatrixPoint {
                    temp: 65f64,
                    speed: 69f64,
                },
                MatrixPoint {
                    temp: 70f64,
                    speed: 75f64,
                },
                MatrixPoint {
                    temp: 75f64,
                    speed: 89f64,
                },
                MatrixPoint {
                    temp: 80f64,
                    speed: 100f64,
                },
            ],
            temp_input: Some(TempInput(1)),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Fan speed {value:?} for config entry {index:} is too low (minimal value is 0.0)")]
    FanSpeedTooLow { value: f64, index: usize },
    #[error("Fan speed {value:?} for config entry {index:} is too high (maximal value is 100.0)")]
    FanSpeedTooHigh { value: f64, index: usize },
    #[error(
    "Fan speed {current:?} for config entry {index} is lower than previous value {last:?}. Entries must be sorted"
    )]
    UnsortedFanSpeed {
        current: f64,
        index: usize,
        last: f64,
    },
    #[error(
    "Fan temperature {current:?} for config entry {index} is lower than previous value {last:?}. Entries must be sorted"
    )]
    UnsortedFanTemp {
        current: f64,
        index: usize,
        last: f64,
    },
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

pub fn load_config(config_path: &str) -> Result<Config, ConfigError> {
    let mut config = ensure_config::<Config, ConfigError, _>(config_path)?;
    config.path = String::from(config_path);

    let mut last_point: Option<&MatrixPoint> = None;

    for (index, matrix_point) in config.speed_matrix.iter().enumerate() {
        if matrix_point.speed < 0f64 {
            log::error!("Fan speed can't be below 0.0 found {}", matrix_point.speed);
            return Err(ConfigError::FanSpeedTooLow {
                value: matrix_point.speed,
                index,
            });
        }
        if matrix_point.speed > 100f64 {
            log::error!(
                "Fan speed can't be above 100.0 found {}",
                matrix_point.speed
            );
            return Err(ConfigError::FanSpeedTooHigh {
                value: matrix_point.speed,
                index,
            });
        }
        if let Some(last_point) = last_point {
            if matrix_point.speed < last_point.speed {
                log::error!(
                    "Curve fan speeds should be monotonically increasing, found {} then {}",
                    last_point.speed,
                    matrix_point.speed
                );

                return Err(ConfigError::UnsortedFanSpeed {
                    current: matrix_point.speed,
                    last: last_point.speed,
                    index,
                });
            }
            if matrix_point.temp < last_point.temp {
                log::error!(
                    "Curve fan temps should be monotonically increasing, found {} then {}",
                    last_point.temp,
                    matrix_point.temp
                );

                return Err(ConfigError::UnsortedFanTemp {
                    current: matrix_point.temp,
                    last: last_point.temp,
                    index,
                });
            }
        }

        last_point = Some(matrix_point)
    }

    Ok(config)
}

#[cfg(test)]
mod parse_config {
    use serde::Deserialize;

    use amdgpu::{AmdGpuError, Card, TempInput};

    #[derive(Deserialize, PartialEq, Debug)]
    pub struct Foo {
        card: Card,
    }

    #[test]
    fn parse_card0() {
        assert_eq!("card0".parse::<Card>(), Ok(Card(0)))
    }

    #[test]
    fn parse_card1() {
        assert_eq!("card1".parse::<Card>(), Ok(Card(1)))
    }

    #[test]
    fn toml_card0() {
        assert_eq!(toml::from_str("card = 'card0'"), Ok(Foo { card: Card(0) }))
    }

    #[test]
    fn parse_invalid_temp_input() {
        assert_eq!(
            "".parse::<TempInput>(),
            Err(AmdGpuError::InvalidTempInput("".to_string()))
        );
        assert_eq!(
            "12".parse::<TempInput>(),
            Err(AmdGpuError::InvalidTempInput("12".to_string()))
        );
        assert_eq!(
            "temp12".parse::<TempInput>(),
            Err(AmdGpuError::InvalidTempInput("temp12".to_string()))
        );
        assert_eq!(
            "12_input".parse::<TempInput>(),
            Err(AmdGpuError::InvalidTempInput("12_input".to_string()))
        );
        assert_eq!(
            "temp_12_input".parse::<TempInput>(),
            Err(AmdGpuError::InvalidTempInput("temp_12_input".to_string()))
        );
    }

    #[test]
    fn parse_valid_temp_input() {
        assert_eq!("temp12_input".parse::<TempInput>(), Ok(TempInput(12)));
    }
}

#[cfg(test)]
mod speed_for_temp {
    use super::*;

    #[test]
    fn below_minimal() {
        let config = Config::default();
        assert_eq!(config.speed_for_temp(1f64), 4f64);
    }

    #[test]
    fn minimal() {
        let config = Config::default();
        assert_eq!(config.speed_for_temp(4f64), 4f64);
    }

    #[test]
    fn between_3_and_4_temp_46() {
        let config = Config::default();
        // 45 -> 50
        // 60 -> 66
        assert_eq!(config.speed_for_temp(46f64).round(), 51f64);
    }

    #[test]
    fn between_3_and_4_temp_58() {
        let config = Config::default();
        // 45 -> 50
        // 60 -> 66
        assert_eq!(config.speed_for_temp(58f64).round(), 64f64);
    }

    #[test]
    fn between_3_and_4_temp_59() {
        let config = Config::default();
        // 45 -> 50
        // 60 -> 66
        assert_eq!(config.speed_for_temp(59f64).round(), 65f64);
    }

    #[test]
    fn average() {
        let config = Config::default();
        assert_eq!(config.speed_for_temp(60f64), 66f64);
    }

    #[test]
    fn max() {
        let config = Config::default();
        assert_eq!(config.speed_for_temp(80f64), 100f64);
    }

    #[test]
    fn above_max() {
        let config = Config::default();
        assert_eq!(config.speed_for_temp(160f64), 100f64);
    }
}

#[cfg(test)]
mod serde_tests {
    use crate::fan::Config;

    #[test]
    fn serialize() {
        let res = toml::to_string(&Config::default());
        assert!(res.is_ok());
    }

    #[test]
    fn deserialize() {
        let res = toml::from_str::<Config>(&toml::to_string(&Config::default()).unwrap());
        assert!(res.is_ok());
    }
}
