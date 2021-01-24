//! Contains structure which provides access to Private section of Coinbase api

use futures_util::future::TryFutureExt;
use hyper::header::HeaderValue;
use hyper::{Body, Method, Request, Uri};
use serde_json;
use std::future::Future;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::adapters::{Adapter, AdapterNew};
use crate::error::*;
use crate::structs::private::*;
use crate::structs::reqs;
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;

use crate::public::Public;

// Create alias for HMAC-SHA256
type HmacSha256 = Hmac<Sha256>;

pub struct Private<Adapter> {
    _pub: Public<Adapter>,
    key: String,
    secret: String,
    passphrase: String,
}

impl<A> Private<A> {
    pub fn sign(secret: &str, timestamp: u64, method: Method, uri: &str, body_str: &str) -> String {
        let key = base64::decode(secret).expect("base64::decode secret");
        let mut mac = HmacSha256::new_varkey(&key).expect("Hmac::new(key)");
        mac.update((timestamp.to_string() + method.as_str() + uri + body_str).as_bytes());
        base64::encode(&mac.finalize().into_bytes())
    }

    fn call_feature<U>(
        &self,
        method: Method,
        uri: &str,
        body_str: &str,
    ) -> impl Future<Output = Result<U, CBError>>
    where
        for<'de> U: serde::Deserialize<'de> + 'static,
    {
        self._pub
            .call_future(self.request(method, uri, body_str.to_string()))
    }

    fn call<U>(&self, method: Method, uri: &str, body_str: &str) -> A::Result
    where
        A: Adapter<U> + 'static,
        U: Send + 'static,
        for<'de> U: serde::Deserialize<'de>,
    {
        self._pub
            .call(self.request(method, uri, body_str.to_string()))
    }

    fn call_get<U>(&self, uri: &str) -> A::Result
    where
        A: Adapter<U> + 'static,
        U: Send + 'static,
        for<'de> U: serde::Deserialize<'de>,
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

        let req = Request::builder().method(&method).uri(uri);

        let sign = Self::sign(&self.secret, timestamp, method, _uri, &body_str);

        let req = req
            .header("User-Agent", Public::<A>::USER_AGENT)
            .header("Content-Type", "Application/JSON")
            //        .header("Accept", "*/*")
            .header("CB-ACCESS-KEY", HeaderValue::from_str(&self.key).unwrap())
            .header("CB-ACCESS-SIGN", HeaderValue::from_str(&sign).unwrap())
            .header(
                "CB-ACCESS-TIMESTAMP",
                HeaderValue::from_str(&timestamp.to_string()).unwrap(),
            )
            .header(
                "CB-ACCESS-PASSPHRASE",
                HeaderValue::from_str(&self.passphrase).unwrap(),
            );

        req.body(body_str.into()).unwrap()
    }

    /// Creates a new Private struct
    pub fn new(uri: &str, key: &str, secret: &str, passphrase: &str) -> Self
    where
        A: AdapterNew,
    {
        Self {
            _pub: Public::new(uri),
            key: key.to_string(),
            secret: secret.to_string(),
            passphrase: passphrase.to_string(),
        }
    }

    /// **Get an Account**
    ///
    /// Get a list of trading accounts
    ///
    /// # API Key Permissions
    /// This endpoint requires either the “view” or “trade” permission.
    pub fn get_accounts(&self) -> A::Result
    where
        A: Adapter<Vec<Account>> + 'static,
    {
        self.call_get("/accounts")
    }

    /// **Get Account History**
    ///
    /// Information for a single account. Use this endpoint when you know the account_id.
    ///
    /// # API Key Permissions
    /// This endpoint requires either the “view” or “trade” permission.
    ///
    /// # Account Fields
    /// | Field | Description |
    /// | ----- | ----------- |
    /// | id |	Account ID |
    /// | balance |	total funds in the account |
    /// | holds |	funds on hold (not available for use) |
    /// | available |	funds available to withdraw or trade |
    pub fn get_account(&self, account_id: Uuid) -> A::Result
    where
        A: Adapter<Account> + 'static,
    {
        self.call_get(&format!("/accounts/{}", account_id))
    }

    /// **Get Account History**
    /// List account activity. Account activity either increases or decreases your account balance.
    /// Items are paginated and sorted latest first. See the Pagination section for retrieving
    /// additional entries after the first page.
    /// # API Key Permissions
    /// This endpoint requires either the “view” or “trade” permission.
    ///
    /// # Entry Types
    /// | Field | Description |
    /// | ----- | ----------- |
    /// | type |	Entry type indicates the reason for the account change. |
    /// | transfer |	Funds moved to/from Coinbase to Coinbase Pro |
    /// | match |	Funds moved as a result of a trade |
    /// | fee |	Fee as a result of a trade |
    /// | rebate |	Fee rebate as per our fee schedule |
    ///
    /// # Details
    ///
    /// If an entry is the result of a trade (match, fee), the details field will contain additional information about the trade.
    pub fn get_account_hist(&self, id: Uuid) -> A::Result
    where
        A: Adapter<Vec<AccountHistory>> + 'static,
    {
        let f = self
            .call_feature(Method::GET, &format!("/accounts/{}/ledger", id), "")
            .map_ok(|xs: Vec<AccountHistory>| {
                xs.into_iter()
                    .map(|x| AccountHistory {
                        _type: (&x.details).into(),
                        ..x
                    })
                    .collect()
            });

        self._pub.adapter.process(f)
    }

    /// **Get Holds**
    /// Holds are placed on an account for any active orders or pending withdraw requests.
    /// As an order is filled, the hold amount is updated. If an order is canceled, any remaining
    /// hold is removed. For a withdraw, once it is completed, the hold is removed.
    ///
    /// # API Key Permissions
    /// This endpoint requires either the “view” or “trade” permission.
    ///
    /// # Type
    /// The type of the hold will indicate why the hold exists. The hold type is order for holds
    /// related to open orders and transfer for holds related to a withdraw.
    ///
    /// # Ref
    /// The ref field contains the id of the order or transfer which created the hold.
    ///
    pub fn get_account_holds(&self, id: Uuid) -> A::Result
    where
        A: Adapter<Vec<AccountHolds>> + 'static,
    {
        self.call_get(&format!("/accounts/{}/holds", id))
    }

    /// **Make Order**
    /// General function. Can be used to use own generated `Order` structure for order
    pub fn set_order(&self, order: reqs::Order) -> A::Result
    where
        A: Adapter<Order> + 'static,
    {
        let body_str = serde_json::to_string(&order).expect("cannot to_string post body");

        self.call(Method::POST, "/orders", &body_str)
    }

    /// **Buy limit**
    /// Makes Buy limit order
    pub fn buy_limit(&self, product_id: &str, size: f64, price: f64, post_only: bool) -> A::Result
    where
        A: Adapter<Order> + 'static,
    {
        self.set_order(reqs::Order::limit(
            product_id,
            reqs::OrderSide::Buy,
            size,
            price,
            post_only,
        ))
    }

    /// **Sell limit**
    /// Makes Sell limit order
    pub fn sell_limit(&self, product_id: &str, size: f64, price: f64, post_only: bool) -> A::Result
    where
        A: Adapter<Order> + 'static,
    {
        self.set_order(reqs::Order::limit(
            product_id,
            reqs::OrderSide::Sell,
            size,
            price,
            post_only,
        ))
    }

    /// **Buy market**
    /// Makes Buy marker order
    pub fn buy_market(&self, product_id: &str, size: f64) -> A::Result
    where
        A: Adapter<Order> + 'static,
    {
        self.set_order(reqs::Order::market(product_id, reqs::OrderSide::Buy, size))
    }

    /// **Sell market**
    /// Makes Sell marker order
    pub fn sell_market(&self, product_id: &str, size: f64) -> A::Result
    where
        A: Adapter<Order> + 'static,
    {
        self.set_order(reqs::Order::market(product_id, reqs::OrderSide::Sell, size))
    }

    //    pub fn buy<'a>(&self) -> OrderBuilder<'a> {}    // TODO: OrderBuilder

    /// **Cancel an Order**
    ///
    /// Cancel a previously placed order.
    ///
    /// If the order had no matches during its lifetime its record may be purged. This means the order details will not be available with GET /orders/<order-id>.
    /// # API Key Permissions
    /// This endpoint requires the “trade” permission.
    pub fn cancel_order(&self, id: Uuid) -> A::Result
    where
        A: Adapter<Uuid> + 'static,
    {
        let f = self.call_feature(Method::DELETE, dbg!(&format!("/orders/{}", id)), "");

        self._pub.adapter.process(f)
    }

    /// **Cancel all**
    ///
    /// With best effort, cancel all open orders. The response is a list of ids of the canceled orders.
    ///
    /// # API Key Permissions
    /// This endpoint requires the “trade” permission.
    ///
    /// # Query Parameters
    /// | Param |	Default |	Description |
    /// | ----- | --------- | ------------- |
    /// | product_id |	*optional* |	Only cancel orders open for a specific product |
    pub fn cancel_all(&self, product_id: Option<&str>) -> A::Result
    where
        A: Adapter<Vec<Uuid>> + 'static,
    {
        let param = product_id
            .map(|x| format!("?product_id={}", x))
            .unwrap_or_default();

        self.call(Method::DELETE, &format!("/orders{}", param), "")
    }

    /// **List Orders**
    ///
    /// List your current open orders. Only open or un-settled orders are returned.
    /// As soon as an order is no longer open and settled, it will no longer appear in the default request.
    ///
    /// # API Key Permissions
    /// This endpoint requires either the “view” or “trade” permission.
    ///
    /// # Query Parameters
    /// | Param 	Default 	Description |
    /// | ------ | -------- | ------------ |
    /// | status |	*open*, *pending*, *active* | 	Limit list of orders to these statuses. Passing all returns orders of all statuses. |
    /// | product_id |	*optional* |	Only list orders for a specific product |
    pub fn get_orders(&self, status: Option<OrderStatus>, product_id: Option<&str>) -> A::Result
    where
        A: Adapter<Vec<Order>> + 'static,
    {
        // TODO rewrite
        let param_status = status.map(|x| format!("&status={}", x)).unwrap_or_default();
        let param_product = product_id
            .map(|x| format!("&product_id={}", x))
            .unwrap_or_default();
        let mut param = (param_status + &param_product).into_bytes();
        if !param.is_empty() {
            param[0] = b'?';
        }

        self.call_get(&format!("/orders{}", String::from_utf8(param).unwrap()))
    }

    /// **Get an Order**
    ///
    /// Get a single order by order id.
    ///
    /// # API Key Permissions
    /// This endpoint requires either the “view” or “trade” permission.
    ///
    /// If the order is canceled the response may have status code 404 if the order had no matches.
    pub fn get_order(&self, id: Uuid) -> A::Result
    where
        A: Adapter<Order> + 'static,
    {
        self.call_get(&format!("/orders/{}", id))
    }

    /// **List Fills**
    ///
    /// Get a list of recent fills.
    ///
    /// # API Key Permissions
    /// This endpoint requires either the “view” or “trade” permission.
    /// **DEPRECATION NOTICE** - Requests without either order_id or product_id will be rejected after 8/23/18.
    pub fn get_fills(&self, order_id: Option<Uuid>, product_id: Option<&str>) -> A::Result
    where
        A: Adapter<Vec<Fill>> + 'static,
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

    /// **Trailing Volume**
    ///
    /// This request will return your 30-day trailing volume for all products. This is a cached
    /// value that’s calculated every day at midnight UTC.
    ///
    /// #API Key Permissions
    /// This endpoint requires either the “view” or “trade” permission.
    pub fn get_trailing_volume(&self) -> A::Result
    where
        A: Adapter<Vec<TrailingVolume>> + 'static,
    {
        self.call_get("/users/self/trailing-volume")
    }

    pub fn public(&self) -> &Public<A> {
        &self._pub
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        structs::reqs::{self, OrderTimeInForce, OrderTimeInForceCancelAfter},
        utils::delay,
        Sync, SANDBOX_URL,
    };

    static KEY: &str = "9eaa4603717ffdc322771a933ae12501";
    static SECRET: &str =
        "RrLem7Ihmnn57ryW4Cc3Rp31h+Bm2DEPmzNbRiPrQQRE1yH6WNybmhK8xSqHjUNaR/V8huS+JMhBlr8PKt2GhQ==";
    static PASSPHRASE: &str = "sandbox";

    #[test]
    #[serial]
    fn test_get_accounts() {
        delay();
        let client: Private<crate::Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let accounts = client.get_accounts().unwrap();
        assert!(format!("{:?}", accounts).contains(r#"currency: "BTC""#));
        assert!(format!("{:?}", accounts).contains(r#"currency: "ETH""#));
    }

    #[test]
    #[serial]
    fn test_get_account() {
        delay();
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
    #[serial]
    fn test_get_account_hist() {
        delay();
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
        println!("{}", account_str);
        assert!(account_str.contains("type: Match, details: Match"));
    }

    #[test]
    #[serial]
    #[ignore]
    fn test_get_account_holds() {
        delay();
        //        super::super::pretty_env_logger::init_custom_env("RUST_LOG=trace");
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let coin_acc = client
            .get_accounts()
            .unwrap()
            .into_iter()
            .find(|x| x.currency == "USD")
            .unwrap();
        let acc_holds = client.get_account_holds(coin_acc.id);
        let _str = format!("{:?}", acc_holds);
        //        assert!(account_str.contains("transfer_type: Deposit"));
        //println!("{:?}", str);
        assert!(false); // TODO: holds are empty now
    }

    #[test]
    fn test_new_order_ser() {
        let order = reqs::Order::buy_market("BTC-UST", 1.1);
        let str = serde_json::to_string(&order).unwrap();
        assert_eq!(
            vec![0],
            str.match_indices("{").map(|(x, _)| x).collect::<Vec<_>>()
        );
    }

    #[test]
    #[serial]
    #[ignore] // sandbox price is too high
    fn test_set_order_limit() {
        delay();
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let order = client.buy_limit("BTC-USD", 1.0, 1.12, true).unwrap();
        let str = format!("{:?}", order);
        assert!(str.contains("side: Buy"));
        assert!(str.contains("_type: Limit {"));
        let order = client.sell_limit("BTC-USD", 0.001, 100000.0, true).unwrap();
        let str = format!("{:?}", order);
        assert!(str.contains("side: Sell"));
        assert!(str.contains("_type: Limit {"));
    }

    #[test]
    #[serial]
    fn test_set_order_limit_gtc() {
        delay();
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);

        let order = reqs::Order::buy_limit("BTC-USD", 1.0, 1.12, true).time_in_force(
            OrderTimeInForce::GTT {
                cancel_after: OrderTimeInForceCancelAfter::Min,
            },
        );

        let order = client.set_order(order).unwrap();
        //        let order = client.buy("BTC-USD", 1.0).limit(1.0, 1.12).post_only().gtt(min).send()
        let str = format!("{:?}", order);
        assert!(str.contains("time_in_force: GTT { expire_time: 2"));
    }

    #[test]
    #[serial]
    fn test_set_order_stop() {
        delay();
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);

        let order = reqs::Order::buy_limit("BTC-USD", 1.0, 1.12, false)
            .stop_entry(0.8)
            .time_in_force(OrderTimeInForce::GTT {
                cancel_after: OrderTimeInForceCancelAfter::Min,
            });

        let str = serde_json::to_string(&order).unwrap();
        assert!(str.contains("stop_price\":0.8,\"stop\":\"entry\""));

        let order = client.set_order(order).unwrap();
        assert!(order.stop.is_none());
    }

    #[test]
    #[serial]
    #[ignore] // sandbox price is too high
    fn test_set_order_market() {
        delay();
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let order = client.buy_market("BTC-USD", 0.001).unwrap();
        let str = format!("{:?}", order);
        assert!(str.contains("side: Buy"));
        assert!(str.contains("_type: Market {"));
        let order = client.sell_market("BTC-USD", 0.001).unwrap();
        let str = format!("{:?}", order);
        assert!(str.contains("side: Sell"));
        assert!(str.contains("_type: Market {"));
        assert!(false);
    }

    #[test]
    #[serial]
    fn test_cancel_order() {
        delay();
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let order = client.buy_limit("BTC-USD", 1.0, 1.12, true).unwrap();
        delay();
        let res = client.cancel_order(order.id).unwrap();
        assert_eq!(order.id, res);
    }

    #[test]
    #[serial]
    fn test_cancel_all() {
        delay();
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let order1 = client.buy_limit("BTC-USD", 1.0, 1.12, true).unwrap();
        let order2 = client.buy_limit("BTC-USD", 1.0, 1.12, true).unwrap();
        let res = client.cancel_all(Some("BTC-USD")).unwrap();
        assert!(res.iter().find(|x| **x == order1.id).is_some());
        assert!(res.iter().find(|x| **x == order2.id).is_some());
    }

    #[test]
    #[serial]
    #[ignore]
    fn test_get_orders() {
        delay();
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let orders = client.get_orders(None, None).unwrap();
        let str = format!("{:?}", orders);
        println!("{}", str);
        assert!(false);
    }

    #[test]
    #[serial]
    fn test_get_order() {
        delay();
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let order = client.buy_limit("BTC-USD", 1.0, 1.12, true).unwrap();
        let order_res = client.get_order(order.id).unwrap();
        assert_eq!(order.id, order_res.id);
    }

    #[test]
    #[serial]
    fn test_get_fills() {
        delay();
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let fills = client.get_fills(None, Some("BTC-USD")).unwrap();
        if fills.len() > 0 {
            let str = format!("{:?}", fills);
            assert!(str.contains("Fill { trade_id: "));
        }
    }

    #[test]
    #[serial]
    #[ignore]
    fn test_get_trailing_volume() {
        delay();
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let vols = client.get_trailing_volume().unwrap();
        let str = format!("{:?}", vols);
        assert!(str == "[]"); // nothing now
    }

    #[test]
    #[serial]
    fn test_get_pub() {
        delay();
        let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
        let time = client.public().get_time().unwrap();
        let time_str = format!("{:?}", time);
        assert!(time_str.starts_with("Time {"));
        assert!(time_str.contains("iso:"));
        assert!(time_str.contains("epoch:"));
        assert!(time_str.ends_with("}"));
    }
}
