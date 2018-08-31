use utils::f64_from_string;
//use utils::f64_opt_from_string;
use utils::f64_nan_from_string;
use super::DateTime;

#[derive(Serialize, Deserialize, Debug)]
pub struct Subscribe {
    #[serde(rename = "type")]
    pub _type: SubscribeCmd,
    pub product_ids: Vec<String>,
    pub channels: Vec<Channel>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SubscribeCmd {
    Subscribe
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Channel {
    Name (ChannelType),
    WithProduct {
        name: ChannelType,
        product_ids: Vec<String>
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ChannelType {
    Heartbeat,
    Ticker,
    Level2,
    User,
    Matches
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Message {
    Subscriptions {
        channels: Vec<Channel>
    },
    Heartbeat {
        sequence: usize,
        last_trade_id: usize,
        product_id: String,
        time: DateTime
    },
    Ticker(TickerType),
    Level2(Level2Type),
    Error {
        message: String
    },
    InternalError(super::super::error::WSError) // in futures 0.3 probably TryStream
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum TickerType {
    Full {
        trade_id: usize,
        sequence: usize,
        time: DateTime,
        product_id: String,
        #[serde(deserialize_with = "f64_from_string")]
        price: f64,
        side: super::reqs::OrderSide,
        #[serde(deserialize_with = "f64_from_string")]
        last_size: f64,
        #[serde(deserialize_with = "f64_nan_from_string")]
        best_bid: f64,
        #[serde(deserialize_with = "f64_nan_from_string")]
        best_ask: f64
    },
    Empty {
        sequence: usize,
        product_id: String,
        #[serde(deserialize_with = "f64_nan_from_string")]
        price: f64
    },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Level2Type {
    Snapshot {
        product_id: String,
    },
    L2update {
        product_id: String,
    }
}

