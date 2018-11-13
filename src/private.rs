//! Contains structure which provides access to Private section of Coinbase api

extern crate base64;
extern crate hmac;
extern crate serde;
extern crate sha2;
extern crate tokio;

use hyper::header::HeaderValue;
use hyper::rt::Future;
use hyper::{Body, Method, Request, Uri};
use private::hmac::{Hmac, Mac};
use serde_json;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use adapters::Adapter;
use error::*;
use structs::private::*;
use structs::reqs;

use public::Public;

pub struct Private<Adapter> {
    _pub: Public<Adapter>,
    key: String,
    secret: String,
    passphrase: String,
}

impl<A> Private<A> {
    pub fn sign(secret: &str, timestamp: u64, method: Method, uri: &str, body_str: &str) -> String {
        let key = base64::decode(secret).expect("base64::decode secret");
        let mut mac: Hmac<sha2::Sha256> = Hmac::new_varkey(&key).expect("Hmac::new(key)");
        mac.input((timestamp.to_string() + method.as_str() + uri + body_str).as_bytes());
        base64::encode(&mac.result().code())
    }

    fn call_feature<U>(
        &self,
        method: Method,
        uri: &str,
        body_str: &str,
    ) -> impl Future<Item = U, Error = CBError>
    where
        for<'de> U: serde::Deserialize<'de>,
    {
        self._pub
            .call_feature(self.request(method, uri, body_str.to_string()))
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

        let mut req = Request::builder();
        req.method(&method);
        req.uri(uri);

        let sign = Self::sign(&self.secret, timestamp, method, _uri, &body_str);

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
            .map(|xs: Vec<AccountHistory>| {
                xs.into_iter()
                    .map(|x| AccountHistory {
                        _type: (&x.details).into(),
                        ..x
                    }).collect()
            });

        A::process(f)
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
    pub fn buy_limit(
        &self,
        product_id: &str,
        size: f64,
        price: f64,
        post_only: bool,
        time_in_force: Option<reqs::OrderTimeInForce>,
    ) -> A::Result
    where
        A: Adapter<Order> + 'static,
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

    /// **Sell limit**
    /// Makes Sell limit order
    pub fn sell_limit(
        &self,
        product_id: &str,
        size: f64,
        price: f64,
        post_only: bool,
        time_in_force: Option<reqs::OrderTimeInForce>,
    ) -> A::Result
    where
        A: Adapter<Order> + 'static,
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
        let f = self
            .call_feature(Method::DELETE, &format!("/orders/{}", id), "")
            .map(|r: Vec<Uuid>| *r.first().unwrap());

        A::process(f)
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
