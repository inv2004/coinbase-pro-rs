use super::DateTime;
use serde_json::Value;
use std::fmt;
use utils::f64_from_string;
use utils::usize_from_string;
use uuid::Uuid;

// Public

#[derive(Serialize, Deserialize, Debug)]
pub struct Time {
    pub iso: String,
    pub epoch: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Currency {
    pub id: String,
    pub name: String,
    #[serde(deserialize_with = "f64_from_string")]
    pub min_size: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Product {
    pub id: String,
    base_currency: String,
    quote_currency: String,
    #[serde(deserialize_with = "f64_from_string")]
    base_min_size: f64,
    #[serde(deserialize_with = "f64_from_string")]
    base_max_size: f64,
    #[serde(deserialize_with = "f64_from_string")]
    quote_increment: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Book<T> {
    pub sequence: usize,
    pub bids: Vec<T>,
    pub asks: Vec<T>,
}

pub trait BookLevel {
    fn level() -> u8;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BookRecordL1(
    #[serde(deserialize_with = "f64_from_string")] f64,
    #[serde(deserialize_with = "f64_from_string")] f64,
    usize,
);

impl BookLevel for BookRecordL1 {
    fn level() -> u8 {
        1
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BookRecordL2(
    #[serde(deserialize_with = "f64_from_string")] f64,
    #[serde(deserialize_with = "f64_from_string")] f64,
    usize,
);

impl BookLevel for BookRecordL2 {
    fn level() -> u8 {
        2
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BookRecordL3(
    #[serde(deserialize_with = "f64_from_string")] f64,
    #[serde(deserialize_with = "f64_from_string")] f64,
    Uuid,
);

impl BookLevel for BookRecordL3 {
    fn level() -> u8 {
        3
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ticker {
    trade_id: usize,
    #[serde(deserialize_with = "f64_from_string")]
    price: f64,
    #[serde(deserialize_with = "f64_from_string")]
    size: f64,
    #[serde(deserialize_with = "f64_from_string")]
    bid: f64,
    #[serde(deserialize_with = "f64_from_string")]
    ask: f64,
    #[serde(deserialize_with = "f64_from_string")]
    volume: f64,
    time: DateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Trade {
    time: DateTime,
    trade_id: usize,
    #[serde(deserialize_with = "f64_from_string")]
    price: f64,
    #[serde(deserialize_with = "f64_from_string")]
    size: f64,
    side: super::reqs::OrderSide,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Candle(
    pub usize, // time
    f64,       // low
    f64,       // high
    f64,       // open
    f64,       // close
    f64,       // volume
);

#[derive(Serialize, Deserialize, Debug)]
pub struct Stats24H {
    #[serde(deserialize_with = "f64_from_string")]
    open: f64,
    #[serde(deserialize_with = "f64_from_string")]
    high: f64,
    #[serde(deserialize_with = "f64_from_string")]
    low: f64,
    #[serde(deserialize_with = "f64_from_string")]
    volume: f64,
}

pub enum Granularity {
    M1 = 60,
    M5 = 300,
    M15 = 900,
    H1 = 3600,
    H6 = 21600,
    D1 = 86400,
}
