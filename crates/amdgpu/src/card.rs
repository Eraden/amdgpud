use serde::Deserialize;

use crate::AmdGpuError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Card(pub u32);

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("card{}", self.0))
    }
}

impl std::str::FromStr for Card {
    type Err = AmdGpuError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if !value.starts_with("card") {
            return Err(AmdGpuError::CardInvalidPrefix);
        }
        if value.len() < 5 {
            return Err(AmdGpuError::CardInputTooShort);
        }
        value[4..]
            .parse::<u32>()
            .map_err(|e| AmdGpuError::CardInvalidSuffix(format!("{:?}", e)))
            .map(Card)
    }
}

impl<'de> Deserialize<'de> for Card {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
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
                    Err(AmdGpuError::CardInvalidPrefix) => {
                        Err(E::custom(format!("expect cardX but got {}", value)))
                    }
                    Err(AmdGpuError::CardInvalidSuffix(s)) => Err(E::custom(s)),
                    Err(AmdGpuError::CardInputTooShort) => Err(E::custom(format!(
                        "{:?} must have at least 5 characters",
                        value
                    ))),
                    _ => unreachable!(),
                }
            }
        }
        deserializer.deserialize_str(CardVisitor).map(Card)
    }
}

impl serde::Serialize for Card {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
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
