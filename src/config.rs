use crate::{AmdFanError, CONFIG_PATH};
use log::LevelFilter;
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
            .map(|n| Card(n))
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
                    Ok(card) => Ok(card.0),
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
                }
            }
        }
        deserializer.deserialize_str(CardVisitor).map(|v| Card(v))
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MatrixPoint {
    pub temp: f64,
    pub speed: u32,
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

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Off => LevelFilter::Off,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    log_level: LogLevel,
    cards: Vec<Card>,
    speed_matrix: Vec<MatrixPoint>,
}

impl Config {
    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    pub fn speed_for_temp(&self, temp: f64) -> u32 {
        let idx = match self.speed_matrix.iter().rposition(|p| p.temp <= temp) {
            Some(idx) => idx,
            _ => return 4,
        };

        match (idx, self.speed_matrix.len() - 1) {
            (0, _) => self.min_speed(),
            (current, max) if current == max => self.max_speed(),
            _ => {
                if self.is_exact_point(idx, temp) {
                    return self.speed_matrix.get(idx).map(|p| p.speed).unwrap_or(4);
                }
                let max = match self.speed_matrix.get(idx + 1) {
                    Some(p) => p,
                    _ => return 4,
                };
                let min = match self.speed_matrix.get(idx) {
                    Some(p) => p,
                    _ => return 4,
                };
                let speed_diff = max.speed as f64 - min.speed as f64;
                let temp_diff = max.temp as f64 - min.temp as f64;
                let increase_by =
                    (((temp as f64 - min.temp as f64) / temp_diff) * speed_diff).round();
                min.speed + increase_by as u32
            }
        }
    }

    pub fn log_level(&self) -> LogLevel {
        self.log_level
    }

    fn min_speed(&self) -> u32 {
        self.speed_matrix.first().map(|p| p.speed).unwrap_or(4)
    }

    fn max_speed(&self) -> u32 {
        self.speed_matrix.last().map(|p| p.speed).unwrap_or(100)
    }

    fn is_exact_point(&self, idx: usize, temp: f64) -> bool {
        static DELTA: f64 = 0.001f64;
        self.speed_matrix
            .get(idx)
            .map(|p| p.temp - DELTA < temp && p.temp + DELTA > temp)
            .unwrap_or(false)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log_level: LogLevel::Error,
            cards: vec![Card(0)],
            speed_matrix: vec![
                MatrixPoint {
                    temp: 4f64,
                    speed: 4,
                },
                MatrixPoint {
                    temp: 30f64,
                    speed: 33,
                },
                MatrixPoint {
                    temp: 45f64,
                    speed: 50,
                },
                MatrixPoint {
                    temp: 60f64,
                    speed: 66,
                },
                MatrixPoint {
                    temp: 65f64,
                    speed: 69,
                },
                MatrixPoint {
                    temp: 70f64,
                    speed: 75,
                },
                MatrixPoint {
                    temp: 75f64,
                    speed: 89,
                },
                MatrixPoint {
                    temp: 80f64,
                    speed: 100,
                },
            ],
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

    if config.speed_matrix.iter().fold(
        1000,
        |n, point| if point.speed < n { point.speed } else { n },
    ) < 4
    {
        log::error!("Due to driver bug lowest fan speed must be greater or equal 4");
        return Err(std::io::Error::from(ErrorKind::InvalidData));
    }

    let mut last_point: Option<&MatrixPoint> = None;

    for matrix_point in config.speed_matrix.iter() {
        if matrix_point.speed <= 0 {
            log::error!("Fan speed can's be below 0 found {}", matrix_point.speed);
            return Err(std::io::Error::from(ErrorKind::InvalidData));
        }
        if matrix_point.speed > 100 {
            log::error!("Fan speed can's be above 100 found {}", matrix_point.speed);
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
        assert_eq!(config.speed_for_temp(1f64), 4);
    }

    #[test]
    fn minimal() {
        let config = Config::default();
        assert_eq!(config.speed_for_temp(4f64), 4);
    }

    #[test]
    fn between_3_and_4_temp_46() {
        let config = Config::default();
        // 45 -> 50
        // 60 -> 66
        assert_eq!(config.speed_for_temp(46f64), 51);
    }

    #[test]
    fn between_3_and_4_temp_58() {
        let config = Config::default();
        // 45 -> 50
        // 60 -> 66
        assert_eq!(config.speed_for_temp(58f64), 64);
    }

    #[test]
    fn between_3_and_4_temp_59() {
        let config = Config::default();
        // 45 -> 50
        // 60 -> 66
        assert_eq!(config.speed_for_temp(59f64), 65);
    }

    #[test]
    fn average() {
        let config = Config::default();
        assert_eq!(config.speed_for_temp(60f64), 66);
    }

    #[test]
    fn max() {
        let config = Config::default();
        assert_eq!(config.speed_for_temp(80f64), 100);
    }

    #[test]
    fn above_max() {
        let config = Config::default();
        assert_eq!(config.speed_for_temp(160f64), 100);
    }
}
