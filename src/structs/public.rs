use super::DateTime;
use crate::utils::f64_from_string;
use serde::{Deserialize, Serialize};
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
    pub display_name: String,
    pub base_currency: String,
    pub quote_currency: String,
    #[serde(deserialize_with = "f64_from_string")]
    pub base_increment: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub quote_increment: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub base_min_size: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub base_max_size: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub min_market_funds: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub max_market_funds: f64,
    pub status: String,
    pub status_message: String,
    pub cancel_only: bool,
    pub limit_only: bool,
    pub post_only: bool,
    pub trading_disabled: bool,
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
pub struct BookRecordL1 {
    #[serde(deserialize_with = "f64_from_string")]
    pub price: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub size: f64,
    pub num_orders: usize,
}

impl BookLevel for BookRecordL1 {
    fn level() -> u8 {
        1
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BookRecordL2 {
    #[serde(deserialize_with = "f64_from_string")]
    pub price: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub size: f64,
    pub num_orders: usize,
}

impl BookLevel for BookRecordL2 {
    fn level() -> u8 {
        2
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BookRecordL3 {
    #[serde(deserialize_with = "f64_from_string")]
    pub price: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub size: f64,
    pub order_id: Uuid,
}

impl BookLevel for BookRecordL3 {
    fn level() -> u8 {
        3
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ticker {
    pub trade_id: usize,
    #[serde(deserialize_with = "f64_from_string")]
    pub price: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub size: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub bid: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub ask: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub volume: f64,
    pub time: DateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Trade {
    pub time: DateTime,
    pub trade_id: usize,
    #[serde(deserialize_with = "f64_from_string")]
    pub price: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub size: f64,
    pub side: super::reqs::OrderSide,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Candle(
    pub usize, // time
    pub f64,   // low
    pub f64,   // high
    pub f64,   // open
    pub f64,   // close
    pub f64,   // volume
);

#[derive(Serialize, Deserialize, Debug)]
pub struct Stats24H {
    #[serde(deserialize_with = "f64_from_string")]
    pub open: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub high: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub low: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub volume: f64,
}

pub enum Granularity {
    M1 = 60,
    M5 = 300,
    M15 = 900,
    H1 = 3600,
    H4 = 14400,
    H6 = 21600,
    D1 = 86400,
}
