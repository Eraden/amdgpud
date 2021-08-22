use crate::{AmdFanError, CONFIG_PATH};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;
use std::io::ErrorKind;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Card(pub u32);

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("card{}", self.0))
    }
}

impl FromStr for Card {
    type Err = AmdFanError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if !value.starts_with("card") {
            return Err(AmdFanError::InvalidPrefix);
        }
        if value.len() < 5 {
            return Err(AmdFanError::InputTooShort);
        }
        value[4..]
            .parse::<u32>()
            .map_err(|e| AmdFanError::InvalidSuffix(format!("{:?}", e)))
            .map(Card)
    }
}

impl<'de> Deserialize<'de> for Card {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, Visitor};

        struct CardVisitor;

        impl<'de> Visitor<'de> for CardVisitor {
            type Value = u32;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("must have format cardX")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value.parse::<Card>() {
                    Ok(card) => Ok(*card),
                    Err(AmdFanError::InvalidPrefix) => {
                        Err(E::custom(format!("expect cardX but got {}", value)))
                    }
                    Err(AmdFanError::InvalidSuffix(s)) => Err(E::custom(s)),
                    Err(AmdFanError::InputTooShort) => Err(E::custom(format!(
                        "{:?} must have at least 5 characters",
                        value
                    ))),
                    Err(AmdFanError::NotAmdCard) => {
                        Err(E::custom(format!("{} is not an AMD GPU", value)))
                    }
                    Err(AmdFanError::FailedReadVendor) => Err(E::custom(format!(
                        "Failed to read vendor file for {}",
                        value
                    ))),
                    _ => unreachable!(),
                }
            }
        }
        deserializer.deserialize_str(CardVisitor).map(Card)
    }
}

impl Serialize for Card {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl std::ops::Deref for Card {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MatrixPoint {
    pub temp: f64,
    pub speed: f64,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum LogLevel {
    /// A level lower than all log levels.
    Off,
    /// Corresponds to the `Error` log level.
    Error,
    /// Corresponds to the `Warn` log level.
    Warn,
    /// Corresponds to the `Info` log level.
    Info,
    /// Corresponds to the `Debug` log level.
    Debug,
    /// Corresponds to the `Trace` log level.
    Trace,
}

impl LogLevel {
    pub fn as_str(&self) -> &str {
        match self {
            LogLevel::Off => "OFF",
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    log_level: LogLevel,
    speed_matrix: Vec<MatrixPoint>,
    temp_input: Option<String>,
}

impl Config {
    pub fn speed_for_temp(&self, temp: f64) -> f64 {
        let idx = match self.speed_matrix.iter().rposition(|p| p.temp <= temp) {
            Some(idx) => idx,
            _ => return self.min_speed(),
        };

        if idx == self.speed_matrix.len() - 1 {
            return self.max_speed();
        }

        crate::utils::linear_map(
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

    pub fn temp_input(&self) -> Option<&str> {
        self.temp_input.as_deref()
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
            temp_input: Some(String::from("temp1_input")),
        }
    }
}

pub fn load_config() -> std::io::Result<Config> {
    let config = match std::fs::read_to_string(CONFIG_PATH) {
        Ok(s) => toml::from_str(&s).unwrap(),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            let config = Config::default();
            std::fs::write(CONFIG_PATH, toml::to_string(&config).unwrap())?;
            config
        }
        Err(e) => {
            log::error!("{:?}", e);
            panic!();
        }
    };

    let mut last_point: Option<&MatrixPoint> = None;

    for matrix_point in config.speed_matrix.iter() {
        if matrix_point.speed < 0f64 {
            log::error!("Fan speed can't be below 0.0 found {}", matrix_point.speed);
            return Err(std::io::Error::from(ErrorKind::InvalidData));
        }
        if matrix_point.speed > 100f64 {
            log::error!(
                "Fan speed can't be above 100.0 found {}",
                matrix_point.speed
            );
            return Err(std::io::Error::from(ErrorKind::InvalidData));
        }
        if let Some(last_point) = last_point {
            if matrix_point.speed < last_point.speed {
                log::error!(
                    "Curve fan speeds should be monotonically increasing, found {} then {}",
                    last_point.speed,
                    matrix_point.speed
                );
                return Err(std::io::Error::from(ErrorKind::InvalidData));
            }
            if matrix_point.temp < last_point.temp {
                log::error!(
                    "Curve fan temps should be monotonically increasing, found {} then {}",
                    last_point.temp,
                    matrix_point.temp
                );
                return Err(std::io::Error::from(ErrorKind::InvalidData));
            }
        }

        last_point = Some(matrix_point)
    }

    Ok(config)
}

#[cfg(test)]
mod parse_config {
    use super::*;
    use serde::Deserialize;

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
