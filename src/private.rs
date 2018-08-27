//! Structure to provide access to Private section of Coinbase api

extern crate base64;
extern crate hmac;
extern crate serde;
extern crate sha2;
extern crate tokio;

use hyper::header::HeaderValue;
use hyper::{Body, HeaderMap, Method, Request, Uri};
use hyper::rt::{Future, Stream};
use private::hmac::{Hmac, Mac};
use serde_json::{self, Value};
use std::fmt::Debug;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use super::Result;
use structs::private::*;
use structs::reqs;
use adapters::*;
use adapters::Adapter;
use error::*;

use public::Public;

pub struct Private<Adapter>{
    _pub: Public<Adapter>,
    key: String,
    secret: String,
    passphrase: String,
}

impl<A> Private<A> {
    fn sign(&self, timestamp: u64, method: Method, uri: &str, body_str: &str) -> String {
        let key = base64::decode(&self.secret).expect("base64::decode secret");
        let mut mac: Hmac<sha2::Sha256> = Hmac::new_varkey(&key).expect("Hmac::new(key)");
        mac.input((timestamp.to_string() + method.as_str() + uri + body_str).as_bytes());
        base64::encode(&mac.result().code())
    }

    fn call_feature<U>(&self, method: Method, uri: &str, body_str: &str) -> impl Future<Item = U, Error = CBError>
        where for<'de> U: serde::Deserialize<'de>
    {
        self._pub.call_feature(self.request(method, uri, body_str.to_string()))
    }

    fn call<U>(&self, method: Method, uri: &str, body_str: &str) -> A::Result
        where A: Adapter<U> + 'static,
            U: Send + 'static,
              for<'de> U: serde::Deserialize<'de>
    {
        self._pub.call(self.request(method, uri, body_str.to_string()))
    }

    fn call_get<U>(&self, uri: &str) -> A::Result
        where A: Adapter<U> + 'static,
            U: Send + 'static,
              for<'de> U: serde::Deserialize<'de>
    {
        self.call(Method::GET, uri, "")
    }

    //   from python
    //POST /orders HTTP/1.1
    //Host: localhost:3000
    //User-Agent: python-requests/2.13.0
    //Accept-Encoding: gzip, deflate
    //Accept: */*
    //Connection: keep-alive
    //Content-Length: 92
    //Content-Type: Application/JSON
    //CB-ACCESS-SIGN: Hy8vbkj3r/XoaT46oQveZs8OIl6zX/xRR6lKTSvfxuk=
    //CB-ACCESS-TIMESTAMP: 1535003621.005189
    //CB-ACCESS-KEY: 1d0dc0f7b4e808d430b95d8fed7df3ea
    //CB-ACCESS-PASSPHRASE: sandbox
    //
    //{"product_id": "BTC-USD", "side": "buy", "type": "limit", "price": "100.00", "size": "0.01"}
    fn request(&self, method: Method, _uri: &str, body_str: String) -> Request<Body> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("leap-second")
            .as_secs();

        let uri: Uri = (self._pub.uri.to_string() + _uri).parse().unwrap();

        let mut req = Request::builder();
        req.method(&method);
        req.uri(uri);

        let sign = self.sign(timestamp, method, _uri, &body_str);

        req.header("User-Agent", Public::<A>::USER_AGENT);
        req.header("Content-Type", "Application/JSON");
        //        req.header("Accept", "*/*");
        req.header("CB-ACCESS-KEY", HeaderValue::from_str(&self.key).unwrap());
        req.header("CB-ACCESS-SIGN", HeaderValue::from_str(&sign).unwrap());
        req.header(
            "CB-ACCESS-TIMESTAMP",
            HeaderValue::from_str(&timestamp.to_string()).unwrap(),
        );
        req.header(
            "CB-ACCESS-PASSPHRASE",
            HeaderValue::from_str(&self.passphrase).unwrap(),
        );

        req.body(body_str.into()).unwrap()
    }

    /// Creates a new Private struct
    pub fn new(uri: &str, key: &str, secret: &str, passphrase: &str) -> Self {
        Self {
            _pub: Public::new(uri),
            key: key.to_string(),
            secret: secret.to_string(),
            passphrase: passphrase.to_string(),
        }
    }

    /// Get a list of trading accounts
    ///
    /// 
    pub fn get_accounts(&self) -> A::Result
        where A: Adapter<Vec<Account>> + 'static
    {
        self.call_get("/accounts")
    }

    pub fn get_account(&self, id: Uuid) -> A::Result
        where A: Adapter<Account> + 'static
    {
        self.call_get(&format!("/accounts/{}", id))
    }

    pub fn get_account_hist(&self, id: Uuid) -> A::Result
        where A: Adapter<Vec<AccountHistory>> + 'static
    {
        let f = self.call_feature(Method::GET, &format!("/accounts/{}/ledger", id), "")
            .map(|xs: Vec<AccountHistory>| {
                xs.into_iter()
                    .map(|x| AccountHistory {
                        _type: (&x.details).into(),
                        ..x
                    }).collect()
            });

        A::process(f)
    }

    pub fn get_account_holds(&self, id: Uuid) -> A::Result
        where A: Adapter<Vec<AccountHolds>> + 'static
    {
        self.call_get(&format!("/accounts/{}/holds", id))
    }

    pub fn set_order(&self, order: reqs::Order) -> A::Result
        where A: Adapter<Order> + 'static
    {
        let body_str = serde_json::to_string(&order)
            .expect("cannot to_string post body");

        self.call(Method::POST, "/orders", &body_str)
    }

    pub fn buy_limit(
        &self,
        product_id: &str,
        size: f64,
        price: f64,
        post_only: bool,
        time_in_force: Option<reqs::OrderTimeInForce>,
    ) -> A::Result
        where A: Adapter<Order> + 'static
    {
        self.set_order(reqs::Order::limit(
            product_id,
            reqs::OrderSide::Buy,
            size,
            price,
            post_only,
            time_in_force,
        ))
    }

    pub fn sell_limit(
        &self,
        product_id: &str,
        size: f64,
        price: f64,
        post_only: bool,
        time_in_force: Option<reqs::OrderTimeInForce>,
    ) -> A::Result
        where A: Adapter<Order> + 'static
    {
        self.set_order(reqs::Order::limit(
            product_id,
            reqs::OrderSide::Sell,
            size,
            price,
            post_only,
            time_in_force,
        ))
    }

    pub fn buy_market(&self, product_id: &str, size: f64) -> A::Result
        where A: Adapter<Order> + 'static
    {
        self.set_order(reqs::Order::market(product_id, reqs::OrderSide::Buy, size))
    }

    pub fn sell_market(&self, product_id: &str, size: f64) -> A::Result
        where A: Adapter<Order> + 'static
    {
        self.set_order(reqs::Order::market(product_id, reqs::OrderSide::Sell, size))
    }

    //    pub fn buy<'a>(&self) -> OrderBuilder<'a> {}    // TODO: OrderBuilder

    pub fn cancel_order(&self, id: Uuid) -> A::Result
        where A: Adapter<Uuid> + 'static
    {
        let f = self.call_feature(Method::DELETE, &format!("/orders/{}", id), "")
           .map(|r: Vec<Uuid>| *r.first().unwrap());

        A::process(f)
    }

    pub fn cancel_all(&self, product_id: Option<&str>) -> A::Result
        where A: Adapter<Vec<Uuid>> + 'static
    {
        let param = product_id
            .map(|x| format!("?product_id={}", x))
            .unwrap_or_default();

        self.call(Method::DELETE, &format!("/orders{}", param), "")
    }

    pub fn get_orders(
        &self,
        status: Option<OrderStatus>,
        product_id: Option<&str>,
    ) -> A::Result
        where A: Adapter<Vec<Order>> + 'static
    {
        // TODO rewrite
        let param_status = product_id
            .map(|x| format!("&product_id={}", x))
            .unwrap_or_default();
        let param_product = product_id
            .map(|x| format!("&product_id={}", x))
            .unwrap_or_default();
        let mut param = (param_status + &param_product).into_bytes();
        if !param.is_empty() {
            param[0] = b'?';
        }

        self.call_get(&format!("/orders{}", String::from_utf8(param).unwrap()))
    }

    pub fn get_order(&self, id: Uuid) -> A::Result
        where A: Adapter<Order> + 'static
    {
        self.call_get(&format!("/orders/{}", id))
    }

    // DEPRECATION NOTICE - Requests without either order_id or product_id will be rejected after 8/23/18.
    pub fn get_fills(&self, order_id: Option<Uuid>, product_id: Option<&str>) -> A::Result
        where A: Adapter<Vec<Fill>> + 'static
    {
        let param_order = order_id
            .map(|x| format!("&order_id={}", x))
            .unwrap_or_default();
        let param_product = product_id
            .map(|x| format!("&product_id={}", x))
            .unwrap_or_default();
        let mut param = (param_order + &param_product).into_bytes();
        if !param.is_empty() {
            param[0] = b'?';
        }
        self.call_get(&format!("/fills{}", String::from_utf8(param).unwrap()))
    }

    pub fn get_trailing_volume(&self) -> A::Result
        where A: Adapter<Vec<TrailingVolume>> + 'static
    {
        self.call_get("/users/self/trailing-volume")
    }

    pub fn public(&self) -> &Public<A> {
        &self._pub
    }

}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::super::structs::public::*;
    use super::super::structs::reqs;

    static KEY: &str = "1d0dc0f7b4e808d430b95d8fed7df3ea";
    static SECRET: &str =
        "dTUic8DZPqkS77vxhJFEX5IBr13FcFHTzWYOARgT9kDWGdN03uvxBbH/hVy8f4O5RDmuf+9wNpEfhYhw2FCWyA==";
    static PASSPHRASE: &str = "sandbox";

    #[test]
    fn test_get_accounts() {
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let accounts = client.get_accounts().unwrap();
        assert!(
            format!("{:?}", accounts).contains(
                r#"currency: "BCH", balance: 0.0, available: 0.0, hold: 0.0, profile_id: "#
            )
        );
        assert!(
            format!("{:?}", accounts).contains(
                r#"currency: "ETH", balance: 0.0, available: 0.0, hold: 0.0, profile_id: "#
            )
        );
    }

    #[test]
    fn test_get_account() {
        //        super::super::pretty_env_logger::init_custom_env("RUST_LOG=trace");
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let coin_acc = client
            .get_accounts()
            .unwrap()
            .into_iter()
            .find(|x| x.currency == "BTC")
            .unwrap();
        let account = client.get_account(coin_acc.id);
        let account_str = format!("{:?}", account);
        assert!(account_str.contains("id:"));
        assert!(account_str.contains("currency: \"BTC\""));
        assert!(account_str.contains("balance:"));
        assert!(account_str.contains("available:"));
        assert!(account_str.contains("hold:"));
        assert!(account_str.contains("profile_id:"));
    }

    #[test]
    fn test_get_account_hist() {
        //        super::super::pretty_env_logger::init_custom_env("RUST_LOG=trace");
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let coin_acc = client
            .get_accounts()
            .unwrap()
            .into_iter()
            .find(|x| x.currency == "USD")
            .unwrap();
        let account = client.get_account_hist(coin_acc.id);
        let account_str = format!("{:?}", account);
        //        println!("{}", account_str);
        assert!(account_str.contains("type: Match, details: Match"));
    }

    #[test]
    #[ignore]
    fn test_get_account_holds() {
        //        super::super::pretty_env_logger::init_custom_env("RUST_LOG=trace");
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let coin_acc = client
            .get_accounts()
            .unwrap()
            .into_iter()
            .find(|x| x.currency == "USD")
            .unwrap();
        let acc_holds = client.get_account_holds(coin_acc.id);
        let str = format!("{:?}", acc_holds);
        //        assert!(account_str.contains("transfer_type: Deposit"));
        //println!("{:?}", str);
        assert!(false); // TODO: holds are empty now
    }

    #[test]
    fn test_new_order_ser() {
        let order = reqs::Order::market("BTC-UST", reqs::OrderSide::Buy, 1.1);
        let str = serde_json::to_string(&order).unwrap();
        assert_eq!(
            vec![0],
            str.match_indices("{").map(|(x, _)| x).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_set_order_limit() {
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let order = client.buy_limit("BTC-USD", 1.0, 1.12, true, None).unwrap();
        let str = format!("{:?}", order);
        assert!(str.contains("side: Buy"));
        assert!(str.contains("_type: Limit {"));
        let order = client
            .sell_limit("BTC-USD", 0.001, 100000.0, true, None)
            .unwrap();
        let str = format!("{:?}", order);
        assert!(str.contains("side: Sell"));
        assert!(str.contains("_type: Limit {"));
    }

    #[test]
    fn test_set_order_limit_gtc() {
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let order = client
            .buy_limit(
                "BTC-USD",
                1.0,
                1.12,
                true,
                Some(reqs::OrderTimeInForce::GTT {
                    cancel_after: reqs::OrderTimeInForceCancelAfter::Min,
                }),
            ).unwrap();
        //        let order = client.buy("BTC-USD", 1.0).limit(1.0, 1.12).post_only().gtt(min).send()
        let str = format!("{:?}", order);
        assert!(str.contains("time_in_force: GTT { expire_time: 2"));
    }

    #[test]
    fn test_set_order_market() {
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let order = client.buy_market("BTC-USD", 0.001).unwrap();
        let str = format!("{:?}", order);
        assert!(str.contains("side: Buy"));
        assert!(str.contains("_type: Market {"));
        let order = client.sell_market("BTC-USD", 0.001).unwrap();
        let str = format!("{:?}", order);
        assert!(str.contains("side: Sell"));
        assert!(str.contains("_type: Market {"));
    }

    #[test]
    fn test_cancel_order() {
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let order = client.buy_limit("BTC-USD", 1.0, 1.12, true, None).unwrap();
        let res = client.cancel_order(order.id).unwrap();
        assert_eq!(order.id, res);
    }

    #[test]
    fn test_cancel_all() {
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let order1 = client.buy_limit("BTC-USD", 1.0, 1.12, true, None).unwrap();
        let order2 = client.buy_limit("BTC-USD", 1.0, 1.12, true, None).unwrap();
        let res = client.cancel_all(Some("BTC-USD")).unwrap();
        assert!(res.iter().find(|x| **x == order1.id).is_some());
        assert!(res.iter().find(|x| **x == order2.id).is_some());
    }

    #[test]
    #[ignore]
    fn test_get_orders() {
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let orders = client.get_orders(None, None).unwrap();
        let str = format!("{:?}", orders);
        println!("{}", str);
        assert!(false);
    }

    #[test]
    fn test_get_order() {
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let order = client.buy_limit("BTC-USD", 1.0, 1.12, true, None).unwrap();
        let order_res = client.get_order(order.id).unwrap();
        assert_eq!(order.id, order_res.id);
    }

    #[test]
    fn test_get_fills() {
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let fills = client.get_fills(None, Some("BTC-USD")).unwrap();
        let str = format!("{:?}", fills);
        assert!(str.contains("Fill { trade_id: "));
    }

    #[test]
    fn test_get_trailing_volume() {
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let vols = client.get_trailing_volume().unwrap();
        let str = format!("{:?}", vols);
        assert!(str == "[]"); // nothing now
    }

    #[test]
    fn test_get_pub() {
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let time = client.public().get_time().unwrap();
        let time_str = format!("{:?}", time);
        assert!(time_str.starts_with("Time {"));
        assert!(time_str.contains("iso:"));
        assert!(time_str.contains("epoch:"));
        assert!(time_str.ends_with("}"));
    }
}
