use std::iter::Peekable;
use std::str::Chars;

const ENGINE_CLOCK_LABEL: &str = "OD_SCLK:";
const MEMORY_CLOCK_LABEL: &str = "OD_MCLK:";
const CURVE_POINTS_LABEL: &str = "OD_VDDC_CURVE:";

#[derive(Debug, Eq, PartialEq)]
pub struct Frequency {
    pub value: u32,
    pub unit: String,
}

impl ToString for Frequency {
    fn to_string(&self) -> String {
        format!("{}{}", self.value, self.unit)
    }
}

impl std::str::FromStr for Frequency {
    type Err = ClockStateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut buffer = String::with_capacity(8);
        let mut value = None;
        for c in s.trim().chars() {
            if c.is_numeric() && value.is_none() {
                buffer.push(c);
            } else if c.is_numeric() {
                return Err(ClockStateError::NotFrequency(s.to_string()));
            } else if value.is_none() {
                if buffer.is_empty() {
                    return Err(ClockStateError::NotFrequency(s.to_string()));
                }
                value = Some(buffer.parse()?);
                buffer.clear();
                buffer.push(c);
            } else {
                buffer.push(c);
            }
        }
        let value = value.ok_or_else(|| ClockStateError::NotFrequency(s.to_string()))?;
        if !buffer.ends_with("hz") && !buffer.ends_with("Hz") {
            return Err(ClockStateError::NotFrequency(s.to_string()));
        }
        Ok(Self {
            value,
            unit: buffer,
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Voltage {
    pub value: u32,
    pub unit: String,
}

impl ToString for Voltage {
    fn to_string(&self) -> String {
        format!("{}{}", self.value, self.unit)
    }
}

impl std::str::FromStr for Voltage {
    type Err = ClockStateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut buffer = String::with_capacity(8);
        let mut value = None;
        for c in s.trim().chars() {
            if c.is_numeric() && value.is_none() {
                buffer.push(c);
            } else if c.is_numeric() {
                return Err(ClockStateError::NotVoltage(s.to_string()));
            } else if value.is_none() {
                if buffer.is_empty() {
                    return Err(ClockStateError::NotVoltage(s.to_string()));
                }
                value = Some(buffer.parse()?);
                buffer.clear();
                buffer.push(c);
            } else {
                buffer.push(c);
            }
        }
        let value = value.ok_or_else(|| ClockStateError::NotVoltage(s.to_string()))?;
        if !buffer.ends_with('V') {
            return Err(ClockStateError::NotVoltage(s.to_string()));
        }
        Ok(Self {
            value,
            unit: buffer,
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct CurvePoint {
    pub freq: Frequency,
    pub voltage: Voltage,
}

#[derive(Debug, Eq, thiserror::Error, PartialEq)]
pub enum ClockStateError {
    #[error("Can't parse value. {0:?}")]
    ParseValue(#[from] std::num::ParseIntError),
    #[error("Value {0:?} is not a voltage")]
    NotVoltage(String),
    #[error("Value {0:?} is not a frequency")]
    NotFrequency(String),
    #[error("Voltage section for engine clock is not valid. Line {0:?} is malformed")]
    InvalidEngineClockSection(String),
}

#[derive(Debug, Eq, PartialEq)]
pub struct ClockState {
    pub curve_labels: Vec<CurvePoint>,
    pub engine_label_lowest: Option<Frequency>,
    pub engine_label_highest: Option<Frequency>,
    pub memory_label_lowest: Option<Frequency>,
    pub memory_label_highest: Option<Frequency>,
}

impl Default for ClockState {
    fn default() -> Self {
        Self {
            curve_labels: Vec::with_capacity(3),
            engine_label_lowest: None,
            engine_label_highest: None,
            memory_label_lowest: None,
            memory_label_highest: None,
        }
    }
}

impl std::str::FromStr for ClockState {
    type Err = ClockStateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut clock_state = Self::default();
        enum State {
            Unknown,
            ParseEngineClock,
            ParseMemoryClock,
            ParseCurve,
        }
        let mut state = State::Unknown;
        for line in s.lines() {
            let start = match line.chars().position(|c| c != ' ' && c != '\0') {
                Some(idx) => idx,
                _ => continue,
            };

            let line = line[start..].trim();
            match state {
                _ if line == "OD_RANGE:" => break,
                _ if line == ENGINE_CLOCK_LABEL => {
                    state = State::ParseEngineClock;
                }
                _ if line == MEMORY_CLOCK_LABEL => {
                    state = State::ParseMemoryClock;
                }
                _ if line == CURVE_POINTS_LABEL => {
                    state = State::ParseCurve;
                }
                State::ParseEngineClock => {
                    if clock_state.engine_label_lowest.is_none() {
                        clock_state.engine_label_lowest = Some(parse_freq_line(line)?);
                    } else {
                        clock_state.engine_label_highest = Some(parse_freq_line(line)?);
                    }
                }
                State::ParseMemoryClock => {
                    if clock_state.memory_label_lowest.is_none() {
                        clock_state.memory_label_lowest = Some(parse_freq_line(line)?);
                    } else {
                        clock_state.memory_label_highest = Some(parse_freq_line(line)?);
                    }
                }
                State::ParseCurve => {
                    let (freq, volt) = parse_freq_voltage_line(line)?;
                    clock_state.curve_labels.push(CurvePoint {
                        freq,
                        voltage: volt,
                    });
                }
                _ => {}
            }
        }
        Ok(clock_state)
    }
}

fn consume_mode_number<'line>(
    line: &'line str,
    chars: &mut Peekable<Chars<'line>>,
) -> std::result::Result<(), ClockStateError> {
    let mut buffer = String::with_capacity(4);
    while chars.peek().filter(|c| c.is_numeric()).is_some() {
        buffer.push(chars.next().unwrap());
    }
    if buffer.is_empty() {
        return Err(ClockStateError::InvalidEngineClockSection(line.to_string()));
    }
    chars
        .next()
        .filter(|c| *c == ':')
        .ok_or_else(|| ClockStateError::InvalidEngineClockSection(line.to_string()))?;
    Ok(())
}

fn consume_freq(chars: &mut Peekable<Chars>) -> std::result::Result<Frequency, ClockStateError> {
    consume_white(chars);
    chars
        .take_while(|c| *c != ' ')
        .collect::<String>()
        .parse::<Frequency>()
}

fn consume_voltage(chars: &mut Peekable<Chars>) -> std::result::Result<Voltage, ClockStateError> {
    consume_white(chars);
    chars
        .take_while(|c| *c != ' ')
        .collect::<String>()
        .parse::<Voltage>()
}

fn consume_white(chars: &mut Peekable<Chars>) {
    while chars.peek().filter(|c| **c == ' ').is_some() {
        let _ = chars.next();
    }
}

fn parse_freq_line(line: &str) -> std::result::Result<Frequency, ClockStateError> {
    let mut chars = line.chars().peekable();
    consume_mode_number(line, &mut chars)?;
    consume_freq(&mut chars)
}

fn parse_freq_voltage_line(
    line: &str,
) -> std::result::Result<(Frequency, Voltage), ClockStateError> {
    let mut chars = line.chars().peekable();
    consume_mode_number(line, &mut chars)?;
    let freq = consume_freq(&mut chars)?;
    consume_white(&mut chars);
    Ok((freq, consume_voltage(&mut chars)?))
}

#[cfg(test)]
mod parse_frequency {
    use crate::clock_state::{ClockStateError, Frequency};

    #[test]
    fn parse_empty_string() {
        assert_eq!(
            "".parse::<Frequency>(),
            Err(ClockStateError::NotFrequency("".to_string()))
        );
    }

    #[test]
    fn parse_only_v_letter() {
        assert_eq!(
            "v".parse::<Frequency>(),
            Err(ClockStateError::NotFrequency("v".to_string()))
        );
    }

    #[test]
    fn parse_only_hz() {
        assert_eq!(
            "hz".parse::<Frequency>(),
            Err(ClockStateError::NotFrequency("hz".to_string()))
        );
    }

    #[test]
    fn parse_only_mhz() {
        assert_eq!(
            "Mhz".parse::<Frequency>(),
            Err(ClockStateError::NotFrequency("Mhz".to_string()))
        );
    }

    #[test]
    fn parse_0mhz() {
        assert_eq!(
            "0Mhz".parse::<Frequency>(),
            Ok(Frequency {
                value: 0,
                unit: "Mhz".to_string(),
            })
        );
    }

    #[test]
    fn parse_0khz() {
        assert_eq!(
            "0khz".parse::<Frequency>(),
            Ok(Frequency {
                value: 0,
                unit: "khz".to_string(),
            })
        );
    }

    #[test]
    fn parse_0kz() {
        assert_eq!(
            "0hz".parse::<Frequency>(),
            Ok(Frequency {
                value: 0,
                unit: "hz".to_string(),
            })
        );
    }

    #[test]
    fn parse_123mhz() {
        assert_eq!(
            "123Mhz".parse::<Frequency>(),
            Ok(Frequency {
                value: 123,
                unit: "Mhz".to_string(),
            })
        );
    }

    #[test]
    fn parse_123khz() {
        assert_eq!(
            "123khz".parse::<Frequency>(),
            Ok(Frequency {
                value: 123,
                unit: "khz".to_string(),
            })
        );
    }

    #[test]
    fn parse_123kz() {
        assert_eq!(
            "123hz".parse::<Frequency>(),
            Ok(Frequency {
                value: 123,
                unit: "hz".to_string(),
            })
        );
    }
}

#[cfg(test)]
mod state_tests {
    use crate::clock_state::{ClockState, CurvePoint, Frequency, Voltage};

    #[test]
    fn valid_string() {
        let s = r#"
OD_SCLK:
0: 800Mhz
1: 2100Mhz
OD_MCLK:
1: 875MHz
OD_VDDC_CURVE:
0: 800MHz 706mV
1: 1450MHz 772mV
2: 2100MHz 1143mV
OD_RANGE:
SCLK:     800Mhz       2150Mhz
MCLK:     625Mhz        950Mhz
VDDC_CURVE_SCLK[0]:     800Mhz       2150Mhz
VDDC_CURVE_VOLT[0]:     750mV        1200mV
VDDC_CURVE_SCLK[1]:     800Mhz       2150Mhz
VDDC_CURVE_VOLT[1]:     750mV        1200mV
VDDC_CURVE_SCLK[2]:     800Mhz       2150Mhz
VDDC_CURVE_VOLT[2]:     750mV        1200mV
        "#;
        let res = s.trim().parse::<ClockState>();
        assert_eq!(
            res,
            Ok(ClockState {
                curve_labels: vec![
                    CurvePoint {
                        freq: Frequency {
                            value: 800,
                            unit: "MHz".to_string(),
                        },
                        voltage: Voltage {
                            value: 706,
                            unit: "mV".to_string(),
                        },
                    },
                    CurvePoint {
                        freq: Frequency {
                            value: 1450,
                            unit: "MHz".to_string(),
                        },
                        voltage: Voltage {
                            value: 772,
                            unit: "mV".to_string(),
                        },
                    },
                    CurvePoint {
                        freq: Frequency {
                            value: 2100,
                            unit: "MHz".to_string(),
                        },
                        voltage: Voltage {
                            value: 1143,
                            unit: "mV".to_string(),
                        },
                    },
                ],
                engine_label_lowest: Some(Frequency {
                    value: 800,
                    unit: "Mhz".to_string(),
                }),
                engine_label_highest: Some(Frequency {
                    value: 2100,
                    unit: "Mhz".to_string(),
                }),
                memory_label_lowest: Some(Frequency {
                    value: 875,
                    unit: "MHz".to_string(),
                }),
                memory_label_highest: None,
            })
        );
    }
}
