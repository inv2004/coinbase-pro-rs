use std::fmt;
use serde_json::Value;
use uuid::Uuid;
use utils::f64_from_string;
use utils::usize_from_string;
use super::DateTime;

// Public

#[derive(Serialize, Deserialize, Debug)]
pub struct Time {
    pub iso: String,
    pub epoch: f64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Currency {
    pub id: String,
    pub name: String,
    #[serde(deserialize_with = "f64_from_string")]
    pub min_size: f64
}

