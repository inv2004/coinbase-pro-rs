use serde::de::{self, Deserialize, Deserializer, Visitor};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

struct F64InQuotes;

impl<'de> Visitor<'de> for F64InQuotes {
    type Value = f64;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("f64 as a number or string")
    }

    fn visit_f64<E>(self, id: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(id)
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        s.parse().map_err(de::Error::custom)
    }
}

pub fn f64_from_string<'de, D>(d: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_any(F64InQuotes)
}

pub fn f64_opt_from_string<'de, D>(d: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_any(F64InQuotes).map(Some).or(Ok(None))
}

pub fn uuid_opt_from_string<'de, D>(d: D) -> Result<Option<Uuid>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    if s.len() == 0 {
        Ok(None)
    } else {
        Uuid::from_str(&s).map_err(de::Error::custom).map(Some)
    }
}

pub fn f64_nan_from_string<'de, D>(d: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_any(F64InQuotes).or(Ok(std::f64::NAN)) // not sure that 100% correct
}

struct UsizeInQuotes;

impl<'de> Visitor<'de> for UsizeInQuotes {
    type Value = usize;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("usize as a number or string")
    }

    fn visit_u64<E>(self, id: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(id as usize)
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        s.parse().map_err(de::Error::custom)
    }
}

pub fn usize_from_string<'de, D>(d: D) -> Result<usize, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_any(UsizeInQuotes)
}

pub fn datetime_from_string<'de, D>(d: D) -> Result<super::structs::DateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    (s + "").parse().map_err(de::Error::custom)
}

#[cfg(test)]
static DELAY_TIMEOUT: u64 = 200;

#[cfg(test)]
pub fn delay() {
    std::thread::sleep(std::time::Duration::from_millis(DELAY_TIMEOUT));
}
