use serde::{de, Deserialize, Deserializer};

pub fn f64_from_string<'de, D>(d: D) -> Result<f64, D::Error>
    where D: Deserializer<'de>
{
    let s = String::deserialize(d)?;
    s.parse().map_err(de::Error::custom)
}