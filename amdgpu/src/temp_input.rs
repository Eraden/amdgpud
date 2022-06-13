use crate::AmdGpuError;
use serde::Serializer;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct TempInput(pub u16);

impl TempInput {
    pub fn as_string(&self) -> String {
        format!("temp{}_input", self.0)
    }
}

impl std::str::FromStr for TempInput {
    type Err = AmdGpuError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("temp") && s.ends_with("_input") {
            let mut buffer = String::with_capacity(4);
            for c in s[4..].chars() {
                if c.is_numeric() {
                    buffer.push(c);
                } else if buffer.is_empty() {
                    return Err(AmdGpuError::InvalidTempInput(s.to_string()));
                }
            }
            buffer
                .parse()
                .map_err(|e| {
                    log::error!("Temp input error {:?}", e);
                    AmdGpuError::InvalidTempInput(s.to_string())
                })
                .map(Self)
        } else {
            Err(AmdGpuError::InvalidTempInput(s.to_string()))
        }
    }
}

impl serde::Serialize for TempInput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.as_string())
    }
}

impl<'de> serde::Deserialize<'de> for TempInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Visitor};

        struct TempInputVisitor;

        impl<'de> Visitor<'de> for TempInputVisitor {
            type Value = u16;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("must have format cardX")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value.parse::<TempInput>() {
                    Ok(temp) => Ok(temp.0),
                    _ => unreachable!(),
                }
            }
        }
        deserializer
            .deserialize_str(TempInputVisitor)
            .map(|v| TempInput(v as u16))
    }
}
