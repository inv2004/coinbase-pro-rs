#[derive(Serialize, Deserialize, Debug)]
pub struct Order<'a> {
    side: OrderSide,
    product_id: &'a str,
    #[serde(flatten)]
    _type: OrderType,
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
        #[serde(flatten)]
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

impl<'a> Order<'a> {
    pub fn market(product_id: &'a str, side: OrderSide, size: f64) -> Self {
        Order {
            product_id,
            side,
            _type: OrderType::Market{_type: MarketType::Size {size}}
        }
    }

    pub fn limit(product_id: &'a str, side: OrderSide, size: f64, price: f64, post_only: bool) -> Self {
        Order {
            product_id,
            side,
            _type: OrderType::Limit{price, size, post_only}
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "time_in_force")]
pub enum OrderTimeInForce {
    GTC, GTT {
        #[serde(flatten)]
        cancel_after: OrderTimeInForceCancelAfter
    }, IOC, FOK
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum OrderTimeInForceCancelAfter {
    Min, Hour, Day
}

