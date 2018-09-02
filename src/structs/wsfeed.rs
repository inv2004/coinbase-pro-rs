use uuid::Uuid;
use utils::f64_from_string;
use utils::f64_opt_from_string;
use utils::f64_nan_from_string;
use utils::usize_from_string;
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
    Matches,
    Full
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub(crate) enum InputMessage {
    Subscriptions {
        channels: Vec<Channel>
    },
    Heartbeat {
        sequence: usize,
        last_trade_id: usize,
        product_id: String,
        time: DateTime
    },
    Ticker(Ticker),
    Snapshot {
        product_id: String,
        bids: Vec<Level2SnapshotRecord>,
        asks: Vec<Level2SnapshotRecord>
    },
    L2update {
        product_id: String,
        changes: Vec<Level2UpdateRecord>
    },
    LastMatch(Match),
    Received(Received),
    Open(Open),
    Done(Done),
    Match(Match),
    Activate(Activate),
    Change(Change),
    Error {
        message: String
    },
    InternalError(super::super::error::WSError) // in futures 0.3 probably TryStream
}

#[derive(Debug)]
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
    Ticker(Ticker),
    Level2(Level2),
    Match(Match),
    Full(Full),
    Error {
        message: String
    },
    InternalError(super::super::error::WSError) // in futures 0.3 probably TryStream
}

#[derive(Deserialize, Debug)]
pub enum Level2 {
    Snapshot {
        product_id: String,
        bids: Vec<Level2SnapshotRecord>,
        asks: Vec<Level2SnapshotRecord>
    },
    L2update {
        product_id: String,
        changes: Vec<Level2UpdateRecord>
    },
}

#[derive(Deserialize, Debug)]
pub struct Level2SnapshotRecord(
    #[serde(deserialize_with = "f64_from_string")]
    f64,
    #[serde(deserialize_with = "f64_from_string")]
    f64
);

#[derive(Deserialize, Debug)]
pub struct Level2UpdateRecord(
    super::reqs::OrderSide,
    #[serde(deserialize_with = "f64_from_string")]
    f64,
    #[serde(deserialize_with = "f64_from_string")]
    f64
);

#[derive(Deserialize, Debug)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum Ticker {
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
pub enum Full {
    Received(Received),
    Open(Open),
    Done(Done),
    Match(Match),
    Change(Change),
    Activate(Activate)
}

#[derive(Deserialize, Debug)]
#[serde(tag = "order_type")]
#[serde(rename_all = "camelCase")]
pub enum Received {
    Limit {
        time: DateTime,
        product_id: String,
        sequence: usize,
        order_id: Uuid,
        # [serde(deserialize_with = "f64_from_string")]
        size: f64,
        # [serde(deserialize_with = "f64_from_string")]
        price: f64,
        side: super::reqs::OrderSide
    },
    Market {
        time: DateTime,
        product_id: String,
        sequence: usize,
        order_id: Uuid,
        #[serde(default)]
        #[serde(deserialize_with = "f64_opt_from_string")]
        funds: Option<f64>,
        side: super::reqs::OrderSide
    }
}

#[derive(Deserialize, Debug)]
pub struct Open {
    pub time: DateTime,
    pub product_id: String,
    pub sequence: usize,
    pub order_id: Uuid,
    #[serde(deserialize_with = "f64_from_string")]
    pub price: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub remaining_size: f64,
    pub side: super::reqs::OrderSide
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum Done {
    Limit {
        time: DateTime,
        product_id: String,
        sequence: usize,
        #[serde(deserialize_with = "f64_from_string")]
        price: f64,
        order_id: Uuid,
        reason: Reason,
        side: super::reqs::OrderSide,
        #[serde(deserialize_with = "f64_from_string")]
        remaining_size: f64
    },
    Market {
        time: DateTime,
        product_id: String,
        sequence: usize,
        order_id: Uuid,
        reason: Reason,
        side: super::reqs::OrderSide,
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Reason {
    Filled, Canceled
}

#[derive(Deserialize, Debug)]
pub struct Match {
    pub trade_id: usize,
    pub sequence: usize,
    pub maker_order_id: Uuid,
    pub taker_order_id: Uuid,
    pub time: DateTime,
    pub product_id: String,
    #[serde(deserialize_with = "f64_from_string")]
    pub size: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub price: f64,
    pub side: super::reqs::OrderSide
}

#[derive(Deserialize, Debug)]
pub struct Change {
    pub time: DateTime,
    pub sequence: usize,
    pub order_id: Uuid,
    pub product_id: String,
    #[serde(deserialize_with = "f64_from_string")]
    pub new_size: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub old_size: f64,
    #[serde(deserialize_with = "f64_opt_from_string")]
    pub new_funds: Option<f64>,
    #[serde(deserialize_with = "f64_opt_from_string")]
    pub old_funds: Option<f64>,
    #[serde(deserialize_with = "f64_from_string")]
    pub price: f64,
    pub side: super::reqs::OrderSide
}

#[derive(Deserialize, Debug)]
pub struct Activate {
    pub product_id: String,
    #[serde(deserialize_with = "f64_from_string")]
    pub timestamp: f64,
    #[serde(deserialize_with = "usize_from_string")]
    pub user_id: usize,
    pub profile_id: Uuid,
    pub order_id: Uuid,
    pub stop_type: StopType,
    #[serde(deserialize_with = "f64_from_string")]
    pub size: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub funds: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub taker_fee_rate: f64,
    pub private: bool
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum StopType {
    Entry, Exit
}

impl From<InputMessage> for  Message {
    fn from(msg: InputMessage) -> Self {
        match msg {
            InputMessage::Subscriptions {channels} => Message::Subscriptions {channels},
            InputMessage::Heartbeat {sequence, last_trade_id, product_id, time} => Message::Heartbeat {sequence, last_trade_id, product_id, time},
            InputMessage::Ticker(ticker) => Message::Ticker(ticker),
            InputMessage::Snapshot {product_id, bids, asks} => Message::Level2(Level2::Snapshot {product_id, bids, asks}),
            InputMessage::L2update {product_id, changes} => Message::Level2(Level2::L2update {product_id, changes}),
            InputMessage::LastMatch(_match) => Message::Match(_match),
            InputMessage::Received(_match) => Message::Full(Full::Received(_match)),
            InputMessage::Open(open) => Message::Full(Full::Open(open)),
            InputMessage::Done(done) => Message::Full(Full::Done(done)),
            InputMessage::Match(_match) => Message::Full(Full::Match(_match)),
            InputMessage::Change(change) => Message::Full(Full::Change(change)),
            InputMessage::Activate(activate) => Message::Full(Full::Activate(activate)),
            InputMessage::Error {message} => Message::Error {message},
            InputMessage::InternalError(err) => Message::InternalError(err),
        }
    }
}


