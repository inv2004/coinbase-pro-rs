use std::fmt;
use serde_json::Value;
use uuid::Uuid;
use utils::f64_from_string;
use utils::usize_from_string;

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

// Private

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub id: Uuid,
    pub currency: String,
    #[serde(deserialize_with = "f64_from_string")]
    pub balance: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub available: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub hold: f64,
    pub profile_id: Uuid
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountHistory {
    pub id: usize,
    pub created_at: String,
    #[serde(deserialize_with = "f64_from_string")]
    pub amount: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub balance: f64,
    #[serde(skip_deserializing)]
    pub _type: String,
    #[serde(flatten)]
    pub details: AccountHistoryDetails // variants are not not clear
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "details")]
#[serde(rename_all = "camelCase")]
pub enum AccountHistoryDetails {
    Fee {
        order_id: Uuid,
        product_id: String,
        #[serde(deserialize_with = "usize_from_string")]
        trade_id: usize
    },
    Match {
        order_id: Uuid,
        product_id: String,
        #[serde(deserialize_with = "usize_from_string")]
        trade_id: usize
    },
    Rebate {
        order_id: Uuid,
        product_id: String,
        #[serde(deserialize_with = "usize_from_string")]
        trade_id: usize
    },
    Transfer {
        transfer_id: Uuid,
        transfer_type: String
    }
}

impl AccountHistoryDetails {
    pub fn kind_str(&self) -> &str {
        match self {
            AccountHistoryDetails::Fee { .. } => "fee",
            AccountHistoryDetails::Match { .. } => "match",
            AccountHistoryDetails::Transfer { .. } => "transfer",
            AccountHistoryDetails::Rebate { .. } => "rebate"
        }
    }
}

// Messagec
#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    message: String
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
