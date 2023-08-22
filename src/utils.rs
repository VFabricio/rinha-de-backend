use serde::de::{Deserialize, Error as DeserializationError, Unexpected, Visitor};
use std::{
    convert::{AsRef, From},
    fmt,
};

#[derive(Debug)]
pub struct LengthRestrictedString<const MIN: usize, const MAX: usize>(String);

struct LengthRestrictedStringVisitor<const MIN: usize, const MAX: usize>;

impl<'de, const MIN: usize, const MAX: usize> Visitor<'de>
    for LengthRestrictedStringVisitor<MIN, MAX>
{
    type Value = LengthRestrictedString<MIN, MAX>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a string no shorter than {} and no longer than {} characters",
            MIN, MAX
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: DeserializationError,
    {
        if v.len() < MIN || v.len() > MAX {
            Err(E::invalid_value(Unexpected::Str(v), &self))
        } else {
            Ok(LengthRestrictedString(v.to_owned()))
        }
    }
}

impl<'de, const MIN: usize, const MAX: usize> Deserialize<'de>
    for LengthRestrictedString<MIN, MAX>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(LengthRestrictedStringVisitor)
    }
}

impl<const MIN: usize, const MAX: usize> From<LengthRestrictedString<MIN, MAX>> for String {
    fn from(value: LengthRestrictedString<MIN, MAX>) -> Self {
        value.0
    }
}

impl<const MIN: usize, const MAX: usize> AsRef<str> for LengthRestrictedString<MIN, MAX> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
