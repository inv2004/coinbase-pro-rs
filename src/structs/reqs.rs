#[derive(Serialize, Deserialize, Debug)]
pub struct Order {
    side: OrderSide,
    product_id: String,
    #[serde(flatten)]
    t: OrderType,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum OrderSide {
    Buy, Sell
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum OrderType {
    Limit {
        price: f64,
        size: f64,
        post_only: bool
    },
    Market {
        _type: MarketType
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum MarketType {
    Size {
        size: f64
    },
    Funds {
        funds: f64
    }
}

