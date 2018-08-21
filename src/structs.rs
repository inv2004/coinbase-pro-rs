use std::fmt;
use serde_json::Value;
use uuid::Uuid;
use chrono;
use utils::f64_from_string;
use utils::usize_from_string;

type DateTime = chrono::DateTime<chrono::Utc>;

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
    pub created_at: DateTime,
    #[serde(deserialize_with = "f64_from_string")]
    pub amount: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub balance: f64,
    #[serde(skip_deserializing)]
    pub _type: AccountHistoryType,
    #[serde(flatten)]
    pub details: AccountHistoryDetails // variants are not not clear
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AccountHistoryType {
    Fee, Match, Rebate, Transfer, NotSet
}

impl Default for AccountHistoryType {
    fn default() -> Self { AccountHistoryType::NotSet }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
        transfer_type: AccountHistoryDetailsTransferType
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum AccountHistoryDetailsTransferType {
    Deposit, Withdraw
}

impl<'a> From<&'a AccountHistoryDetails> for AccountHistoryType {
    fn from(item: &'a AccountHistoryDetails) -> Self {
        match item {
            AccountHistoryDetails::Fee { .. } => AccountHistoryType::Fee,
            AccountHistoryDetails::Match { .. } => AccountHistoryType::Match,
            AccountHistoryDetails::Transfer { .. } => AccountHistoryType::Transfer,
            AccountHistoryDetails::Rebate { .. } => AccountHistoryType::Rebate
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountHolds {
    pub id: Uuid,
    pub account_id: Uuid,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub amount: f64,
    #[serde(rename = "type")]
    pub _type: AccountHoldsType,
    #[serde(rename = "ref")]
    pub _ref: Uuid
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum AccountHoldsType {
    Order, Transfer
}

// Message
#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    message: String
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

