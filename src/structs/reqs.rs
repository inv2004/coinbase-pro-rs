use std::borrow::Cow;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Order<'a> {
    side: OrderSide,
    client_oid: Option<Uuid>,
    product_id: Cow<'a, str>,
    #[serde(flatten)]
    _type: OrderType,
    #[serde(flatten)]
    stop: Option<OrderStop>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum OrderType {
    Limit {
        price: f64,
        size: f64,
        post_only: bool,
        #[serde(flatten)]
        time_in_force: Option<OrderTimeInForce>,
    },
    Market {
        #[serde(flatten)]
        _type: MarketType,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum MarketType {
    Size { size: f64 },
    Funds { funds: f64 },
}

impl<'a> Order<'a> {
    pub fn market<T: Into<Cow<'a, str>>>(product_id: T, side: OrderSide, size: f64) -> Self {
        Order {
            product_id: product_id.into(),
            client_oid: None,
            side,
            _type: OrderType::Market {
                _type: MarketType::Size { size },
            },
            stop: None,
        }
    }

    pub fn market_with_funds<T: Into<Cow<'a, str>>>(
        product_id: T,
        side: OrderSide,
        funds: f64,
    ) -> Self {
        Order {
            product_id: product_id.into(),
            client_oid: None,
            side,
            _type: OrderType::Market {
                _type: MarketType::Funds { funds },
            },
            stop: None,
        }
    }

    pub fn buy_market<T: Into<Cow<'a, str>>>(product_id: T, size: f64) -> Self {
        Self::market(product_id, OrderSide::Buy, size)
    }
    pub fn buy_market_with_funds<T: Into<Cow<'a, str>>>(product_id: T, funds: f64) -> Self {
        Self::market_with_funds(product_id, OrderSide::Buy, funds)
    }

    pub fn sell_market<T: Into<Cow<'a, str>>>(product_id: T, size: f64) -> Self {
        Self::market(product_id, OrderSide::Sell, size)
    }

    pub fn sell_market_with_funds<T: Into<Cow<'a, str>>>(product_id: T, funds: f64) -> Self {
        Self::market(product_id, OrderSide::Sell, funds)
    }

    pub fn limit<T: Into<Cow<'a, str>>>(
        product_id: T,
        side: OrderSide,
        size: f64,
        price: f64,
        post_only: bool,
    ) -> Self {
        Order {
            product_id: product_id.into(),
            client_oid: None,
            side,
            _type: OrderType::Limit {
                price,
                size,
                post_only,
                time_in_force: None,
            },
            stop: None,
        }
    }

    pub fn buy_limit<T: Into<Cow<'a, str>>>(
        product_id: T,
        size: f64,
        price: f64,
        post_only: bool,
    ) -> Self {
        Self::limit(product_id, OrderSide::Buy, size, price, post_only)
    }

    pub fn sell_limit<T: Into<Cow<'a, str>>>(
        product_id: T,
        size: f64,
        price: f64,
        post_only: bool,
    ) -> Self {
        Self::limit(product_id, OrderSide::Sell, size, price, post_only)
    }

    pub fn client_oid(self, client_oid: Uuid) -> Self {
        let client_oid = Some(client_oid);
        Order { client_oid, ..self }
    }

    pub fn time_in_force(self, time_in_force: OrderTimeInForce) -> Self {
        match self._type {
            OrderType::Limit {
                price,
                size,
                post_only,
                ..
            } => {
                let _type = OrderType::Limit {
                    price,
                    size,
                    post_only,
                    time_in_force: Some(time_in_force),
                };
                Order { _type, ..self }
            }
            _ => panic!("time_in_force is for limit orders only"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "time_in_force")]
pub enum OrderTimeInForce {
    GTC,
    GTT {
        cancel_after: OrderTimeInForceCancelAfter,
    },
    IOC,
    FOK,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum OrderTimeInForceCancelAfter {
    Min,
    Hour,
    Day,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OrderStop {
    stop_price: f64,
    _type: OrderStopType,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum OrderStopType {
    Loss,
    Entry,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_builder() {
        let o = Order::buy_limit("BTC-USD", 10.0, 100.0, true);
        assert!(o.client_oid.is_none());

        match &o._type {
            OrderType::Limit {
                time_in_force: None,
                ..
            } => assert!(true),
            _ => assert!(false),
        }

        let o = Order::buy_limit("BTC-USD", 10.0, 100.0, true)
            .client_oid(Uuid::nil())
            .time_in_force(OrderTimeInForce::GTC);
        assert!(o.client_oid.is_some());

        match &o._type {
            OrderType::Limit {
                time_in_force: Some(OrderTimeInForce::GTC),
                ..
            } => assert!(true),
            _ => assert!(false),
        }
    }

    #[derive(Debug)]
    enum Coin {
        AAA,
        BBB,
    }
    #[derive(Debug)]
    struct Pair {
        a: Coin,
        b: Coin,
    }

    impl<'a> From<Pair> for Order<'a> {
        fn from(pair: Pair) -> Self {
            Order::buy_market(format!("{:?}-{:?}", pair.a, pair.b), 10.0)
        }
    }

    #[test]
    fn test_order_from() {
        let p = Pair {
            a: Coin::AAA,
            b: Coin::BBB,
        };
        let order_owned: Order = p.into();
        assert_eq!(order_owned.product_id, "AAA-BBB");
        let order_str: Order = Order::buy_market("AAA-BBB", 10.0);
        assert_eq!(order_str.product_id, "AAA-BBB");
    }
}
