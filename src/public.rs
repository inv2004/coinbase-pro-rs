extern crate serde;
extern crate serde_json;
extern crate tokio;

use hyper::client::HttpConnector;
use hyper::rt::{Future, Stream};
use hyper::{Body, Client, HeaderMap, Request, Uri};
use hyper_tls::HttpsConnector;
use serde::Deserialize;
use std::fmt::Debug;
use std::marker::PhantomData;

use super::Result;
use error::*;
use super::adapters::*;
use structs::public::*;
use structs::DateTime;

pub struct Public<Adapter> {
    pub uri: String,
    client: Client<HttpsConnector<HttpConnector>>,
    adapter: PhantomData<Adapter>
}

impl<A> Public<A> {
    pub const USER_AGENT: &'static str = "coinbase-pro-rs/0.1.0";

    fn request(&self, uri: &str) -> Request<Body> {
        let uri: Uri = (self.uri.to_string() + uri).parse().unwrap();

        let mut req = Request::get(uri);
        req.header("User-Agent", Self::USER_AGENT);
        req.body(Body::empty()).unwrap()
    }

    pub fn call_feature<U>(&self, request: Request<Body>) -> impl Future<Item = U, Error = CBError>
        where for<'de> U: serde::Deserialize<'de>,
    {
        debug!("{:?}", request);

        self.client
            .request(request)
            .map_err(CBError::Http)
            .and_then(|res| res.into_body().concat2().map_err(CBError::Http))
            .and_then(|body| {
                let res = serde_json::from_slice(&body).map_err(|e| {
                    serde_json::from_slice(&body)
                        .map(CBError::Coinbase)
                        .unwrap_or_else(|_| {
                            let data = String::from_utf8(body.to_vec()).unwrap();
                            CBError::Serde { error: e, data }
                        })
                })?;
                Ok(res)
            })
    }

    pub fn call<U>(&self, request: Request<Body>) -> A::Result
        where
            A: Adapter<U> + 'static,
            U: 'static,
            for<'de> U: serde::Deserialize<'de>,
    {
        A::process(self.call_feature(request))
    }

    pub fn call_get<U>(&self, uri: &str) -> A::Result
        where
            A: Adapter<U> + 'static,
            U: 'static,
            for <'de> U: serde::Deserialize<'de>
    {
        self.call(self.request(uri))
    }

    pub fn new() -> Self {
        let https = HttpsConnector::new(4).unwrap();
        let client = Client::builder().build::<_, Body>(https);
        let uri = "https://api-public.sandbox.pro.coinbase.com".to_string();

        Self { uri, client, adapter: PhantomData }
    }

    pub fn get_time(&self) -> A::Result
        where A: Adapter<Time> + 'static
    {
        self.call_get("/time")
    }

    pub fn get_products(&self) -> A::Result
        where A: Adapter<Vec<Product>> + 'static
    {
        self.call_get("/products")
    }

    pub fn get_book<T>(&self, product_id: &str) -> A::Result
        where A: Adapter<Book<T>> + 'static,
              T: BookLevel + Debug + 'static,
              T: super::std::marker::Send,
              T: for<'de> Deserialize<'de>
    {
        self.call_get(&format!(
            "/products/{}/book?level={}",
            product_id,
            T::level()
        ))
    }

    pub fn get_ticker(&self, product_id: &str) -> A::Result
        where A: Adapter<Ticker> + 'static
    {
        self.call_get(&format!("/products/{}/ticker", product_id))
    }

    pub fn get_trades(&self, product_id: &str) -> A::Result
        where A: Adapter<Vec<Trade>> + 'static
    {
        self.call_get(&format!("/products/{}/trades", product_id))
    }

    pub fn get_candles(
        &self,
        product_id: &str,
        start: Option<DateTime>,
        end: Option<DateTime>,
        granularity: Granularity,
    ) -> A::Result
        where A: Adapter<Vec<Candle>> + 'static
    {
        let param_start = start
            .map(|x| format!("&start={}", x.to_rfc3339()))
            .unwrap_or_default();
        let param_end = end
            .map(|x| format!("&end={}", x.to_rfc3339()))
            .unwrap_or_default();

        let req = format!(
            "/products/{}/candles?granularity={}{}{}",
            product_id, granularity as usize, param_start, param_end
        );
        self.call_get(&req)
    }

    pub fn get_stats24h(&self, product_id: &str) -> A::Result
        where A: Adapter<Stats24H> + 'static
    {

        self.call_get(&format!("/products/{}/stats", product_id))
    }

    pub fn get_currencies(&self) -> A::Result
        where A: Adapter<Vec<Currency>> + 'static
    {
        self.call_get("/currencies")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;
    use time::Duration;

    #[test]
    fn test_get_time() {
        let client: Public<Sync> = Public::new();
        let time = client.get_time().unwrap();
        let time_str = format!("{:?}", time);
        assert!(time_str.starts_with("Time {"));
        assert!(time_str.contains("iso:"));
        assert!(time_str.contains("epoch:"));
        assert!(time_str.ends_with("}"));
    }

    #[test]
    fn test_get_products() {
        let client: Public<Sync> = Public::new();
        let products = client.get_products().unwrap();
        let str = format!("{:?}", products);
        assert!(str.contains("{ id: \"BTC-USD\""));
    }

    #[test]
    fn test_get_book() {
        let client: Public<Sync> = Public::new();
        let book_l1 = client.get_book::<BookRecordL1>("BTC-USD").unwrap();
        let str1 = format!("{:?}", book_l1);
        assert_eq!(1, book_l1.bids.len());
        assert!(str1.contains("bids: [BookRecordL1"));
        let book_l2 = client.get_book::<BookRecordL2>("BTC-USD").unwrap();
        let str2 = format!("{:?}", book_l2);
        assert!(book_l2.bids.len() > 1);
        assert!(str2.contains("[BookRecordL2("));
        let book_l3 = client.get_book::<BookRecordL3>("BTC-USD").unwrap();
        let str3 = format!("{:?}", book_l3);
        assert!(book_l2.bids.len() > 1);
        assert!(str2.contains("[BookRecordL2("));
    }

    #[test]
    fn test_get_ticker() {
        let client: Public<Sync> = Public::new();
        let ticker = client.get_ticker("BTC-USD").unwrap();
        let str = format!("{:?}", ticker);
        assert!(str.starts_with("Ticker { trade_id:"));
        assert!(str.contains("time:"));
    }

    #[test]
    fn test_get_trades() {
        let client: Public<Sync> = Public::new();
        let trades = client.get_trades("BTC-USD").unwrap();
        assert!(trades.len() > 1);
        let str = format!("{:?}", trades);
        assert!(str.starts_with("[Trade { time: "));
    }

    #[test]
    fn test_get_candles() {
        let client: Public<Sync> = Public::new();
        let end = Utc::now();
        //        let start = end - Duration::minutes(10);
        let candles = client
            .get_candles("BTC-USD", None, Some(end), Granularity::M1)
            .unwrap();
        let str = format!("{:?}", candles);
        //        println!("{}", str);
        assert!(candles[0].0 > candles[1].0);
    }

    #[test]
    fn test_get_stats24h() {
        let client: Public<Sync> = Public::new();
        let stats24h = client.get_stats24h("BTC-USD").unwrap();
        let str = format!("{:?}", stats24h);
        assert!(str.contains("open:"));
        assert!(str.contains("high:"));
        assert!(str.contains("low:"));
        assert!(str.contains("volume:"));
    }

    #[test]
    fn test_get_currencies() {
        let client: Public<Sync> = Public::new();
        let currencies = client.get_currencies().unwrap();
        let currency = currencies.iter().find(|x| x.id == "BTC").unwrap();
        assert_eq!(
            format!("{:?}", currency),
            "Currency { id: \"BTC\", name: \"Bitcoin\", min_size: 0.00000001 }"
        );
        let currency = currencies.iter().find(|x| x.id == "LTC").unwrap();
        assert_eq!(
            format!("{:?}", currency),
            "Currency { id: \"LTC\", name: \"Litecoin\", min_size: 0.00000001 }"
        );
    }

    //    #[test]
    //    fn test_tls() { // it hangs
    //        let https = HttpsConnector::new(4).unwrap();
    //        let client = Client::builder()
    //            .build::<_, hyper::Body>(https);
    //        let ft = client
    //            .call("https://hyper.rs".parse().unwrap())
    //            .map_err(|_| ())
    //            .and_then(|res| {
    //                res.into_body().concat2().map_err(|_| ())
    //            })
    //            .and_then(|body| {
    //                println!("body: {:?}", &body);
    //                Ok(())
    //            });
    //        rt::run(ft);
    //    }
}
