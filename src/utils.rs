use std::str::FromStr;
use std::fmt::Display;
use serde::{de, Deserialize, Deserializer};

pub fn from_string<'de, D, T>(d: D) -> Result<T, D::Error>
    where D: Deserializer<'de>,
        T: FromStr,
        <T as FromStr>::Err: Display
{
    let s = String::deserialize(d)?;
    s.parse().map_err(de::Error::custom)
}

pub fn f64_from_string<'de, D>(d: D) -> Result<f64, D::Error>
    where D: Deserializer<'de>
{
    from_string(d)
}

//pub fn f64_opt_from_string<'de, D>(d: D) -> Result<Option<f64>, D::Error>
//    where D: Deserializer<'de>
//{
//    from_string(d).map(Some)
//}

pub fn usize_from_string<'de, D>(d: D) -> Result<usize, D::Error>
    where D: Deserializer<'de>
{
    from_string(d)
}

pub fn datetime_from_string<'de, D>(d: D) -> Result<super::structs::DateTime, D::Error>
    where D: Deserializer<'de>
{
    let s = String::deserialize(d)?;
    (s+"Z").parse().map_err(de::Error::custom)
}

