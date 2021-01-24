//! Contains structure which provides access to Public section of Coinbase api

use chrono::SecondsFormat;
use hyper::client::HttpConnector;
use hyper::{body::to_bytes, Body, Client, Request, Uri};
use hyper_tls::HttpsConnector;
use serde::Deserialize;
use std::fmt::Debug;
use std::future::Future;

use super::adapters::*;
use crate::error::*;
use crate::structs::public::*;
use crate::structs::DateTime;

pub struct Public<Adapter> {
    pub(crate) uri: String,
    pub(crate) adapter: Adapter,
    client: Client<HttpsConnector<HttpConnector>>,
}

impl<A> Public<A> {
    pub(crate) const USER_AGENT: &'static str =
        concat!("coinbase-pro-rs/", env!("CARGO_PKG_VERSION"));

    fn request(&self, uri: &str) -> Request<Body> {
        let uri: Uri = (self.uri.to_string() + uri).parse().unwrap();

        let req = Request::get(uri).header("User-Agent", Self::USER_AGENT);
        req.body(Body::empty()).unwrap()
    }

    fn get_pub<U>(&self, uri: &str) -> A::Result
    where
        A: Adapter<U> + 'static,
        U: Send + 'static,
        for<'de> U: serde::Deserialize<'de>,
    {
        self.call(self.request(uri))
    }

    pub(crate) fn call_future<U>(
        &self,
        request: Request<Body>,
    ) -> impl Future<Output = Result<U, CBError>> + 'static
    where
        for<'de> U: serde::Deserialize<'de> + 'static,
    {
        log::debug!("REQ: {:?}", request);

        let res = self.client.request(request);
        async move {
            let res = res.await.map_err(CBError::Http)?;
            let body = to_bytes(res.into_body()).await.map_err(CBError::Http)?;
            log::debug!("RES: {:#?}", body);
            let res: Result<U, CBError> = serde_json::from_slice(&body).map_err(|e| {
                let err = serde_json::from_slice(&body);
                let err = err.map(CBError::Coinbase).unwrap_or_else(|_| {
                    let data = String::from_utf8(body.to_vec()).unwrap();
                    CBError::Serde { error: e, data }
                });
                err
            });
            res
        }
    }

    pub(crate) fn call<U>(&self, request: Request<Body>) -> A::Result
    where
        A: Adapter<U> + 'static,
        U: Send + 'static,
        for<'de> U: serde::Deserialize<'de>,
    {
        self.adapter.process(self.call_future(request))
    }

    // This function is contructor which can control keep_alive flag of the connection.
    // Created for tests to exit tokio::run
    pub fn new_with_keep_alive(uri: &str, keep_alive: bool) -> Self
    where
        A: AdapterNew,
    {
        let https = HttpsConnector::new();
        let client = Client::builder()
            // Keep this for now
            .keep_alive(keep_alive)
            .build::<_, Body>(https);
        let uri = uri.to_string();

        Self {
            uri,
            client,
            adapter: A::new().expect("Failed to initialize adapter"),
        }
    }

    pub fn new(uri: &str) -> Self
    where
        A: AdapterNew,
    {
        Self::new_with_keep_alive(uri, true)
    }

    pub fn get_time(&self) -> A::Result
    where
        A: Adapter<Time> + 'static,
    {
        self.get_pub("/time")
    }

    pub fn get_products(&self) -> A::Result
    where
        A: Adapter<Vec<Product>> + 'static,
    {
        self.get_pub("/products")
    }

    pub fn get_product(&self, product_id: &str) -> A::Result
    where
        A: Adapter<Product> + 'static,
    {
        self.get_pub(&format!("/products/{}", product_id))
    }

    pub fn get_book<T>(&self, product_id: &str) -> A::Result
    where
        A: Adapter<Book<T>> + 'static,
        T: BookLevel + Debug + 'static,
        T: std::marker::Send,
        T: for<'de> Deserialize<'de>,
    {
        self.get_pub(&format!(
            "/products/{}/book?level={}",
            product_id,
            T::level()
        ))
    }

    pub fn get_ticker(&self, product_id: &str) -> A::Result
    where
        A: Adapter<Ticker> + 'static,
    {
        self.get_pub(&format!("/products/{}/ticker", product_id))
    }

    pub fn get_trades(&self, product_id: &str) -> A::Result
    where
        A: Adapter<Vec<Trade>> + 'static,
    {
        self.get_pub(&format!("/products/{}/trades", product_id))
    }

    pub fn get_candles(
        &self,
        product_id: &str,
        start: Option<DateTime>,
        end: Option<DateTime>,
        granularity: Granularity,
    ) -> A::Result
    where
        A: Adapter<Vec<Candle>> + 'static,
    {
        let param_start = start
            .map(|x| format!("&start={}", x.to_rfc3339_opts(SecondsFormat::Secs, true)))
            .unwrap_or_default();
        let param_end = end
            .map(|x| format!("&end={}", x.to_rfc3339_opts(SecondsFormat::Secs, true)))
            .unwrap_or_default();

        let req = format!(
            "/products/{}/candles?granularity={}{}{}",
            product_id, granularity as usize, param_start, param_end
        );
        self.get_pub(&req)
    }

    pub fn get_stats24h(&self, product_id: &str) -> A::Result
    where
        A: Adapter<Stats24H> + 'static,
    {
        self.get_pub(&format!("/products/{}/stats", product_id))
    }

    pub fn get_currencies(&self) -> A::Result
    where
        A: Adapter<Vec<Currency>> + 'static,
    {
        self.get_pub("/currencies")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use chrono::prelude::*;
    use std::time::Instant;

    static DELAY_TIMEOUT: u64 = 200;

    pub fn delay() {
        std::thread::sleep(std::time::Duration::from_millis(DELAY_TIMEOUT));
    }

    #[test]
    #[serial]
    fn test_get_time() {
        delay();
        let client: Public<Sync> = Public::new(SANDBOX_URL);
        let time = client.get_time().unwrap();
        let time_str = format!("{:?}", time);
        assert!(time_str.starts_with("Time {"));
        assert!(time_str.contains("iso:"));
        assert!(time_str.contains("epoch:"));
        assert!(time_str.ends_with("}"));
    }

    #[test]
    #[serial]
    fn test_get_products() {
        delay();
        let client: Public<Sync> = Public::new(SANDBOX_URL);
        let products = client.get_products().unwrap();
        let str = format!("{:?}", products);
        assert!(str.contains("{ id: \"BTC-USD\""));
    }

    #[test]
    #[serial]
    fn test_get_product() {
        delay();
        let client: Public<Sync> = Public::new(SANDBOX_URL);
        let products = client.get_product("BTC-USD").unwrap();
        let str = format!("{:?}", products);
        assert!(str.contains("{ id: \"BTC-USD\""));
    }

    #[test]
    #[serial]
    #[ignore] // rate limits
    fn test_get_book() {
        delay();
        let client: Public<Sync> = Public::new(SANDBOX_URL);
        let book_l1 = client.get_book::<BookRecordL1>("BTC-USD").unwrap();
        let str1 = format!("{:?}", book_l1);
        assert_eq!(1, book_l1.bids.len());
        assert!(book_l1.bids.len() > 0);
        assert!(book_l1.asks.len() > 0);
        assert!(str1.contains("bids: [BookRecordL1 {"));
        let book_l2 = client.get_book::<BookRecordL2>("BTC-USD").unwrap();
        let str2 = format!("{:?}", book_l2);
        assert!(book_l2.asks.len() > 1);
        assert!(str2.contains("[BookRecordL2 {"));
        let book_l3 = client.get_book::<BookRecordL3>("BTC-USD").unwrap();
        let str3 = format!("{:?}", book_l3);
        assert!(book_l3.asks.len() > 1);
        assert!(str3.contains("[BookRecordL3 {"));
    }

    #[test]
    #[serial]
    fn test_get_ticker() {
        delay();
        let client: Public<Sync> = Public::new(SANDBOX_URL);
        let ticker = client.get_ticker("BTC-USD").unwrap();
        let str = format!("{:?}", ticker);
        dbg!(&str);
        assert!(str.starts_with("Ticker { trade_id:"));
        assert!(str.contains("time:"));
    }

    #[test]
    #[serial]
    fn test_get_trades() {
        delay();
        let client: Public<Sync> = Public::new(SANDBOX_URL);
        let trades = client.get_trades("BTC-USD").unwrap();
        assert!(trades.len() > 1);
        let str = format!("{:?}", trades);
        assert!(str.starts_with("[Trade { time: "));
    }

    #[test]
    #[serial]
    fn test_get_candles() {
        delay();
        let client: Public<Sync> = Public::new(SANDBOX_URL);
        let end = Utc::now();
        //        let start = end - Duration::minutes(10);
        let candles = client
            .get_candles("BTC-USD", None, Some(end), Granularity::M1)
            .unwrap();
        // let str = format!("{:?}", candles);
        // println!("{}", str);
        assert!(candles[0].0 > candles[1].0);
    }

    #[test]
    #[serial]
    fn test_get_stats24h() {
        delay();
        let client: Public<Sync> = Public::new(SANDBOX_URL);
        let stats24h = client.get_stats24h("BTC-USD").unwrap();
        let str = format!("{:?}", stats24h);
        assert!(str.contains("open:"));
        assert!(str.contains("high:"));
        assert!(str.contains("low:"));
        assert!(str.contains("volume:"));
    }

    #[test]
    #[serial]
    fn test_get_currencies() {
        delay();
        let client: Public<Sync> = Public::new(SANDBOX_URL);
        let currencies = client.get_currencies().unwrap();
        println!("{:?}", &currencies);
        let currency = currencies.iter().find(|x| x.id == "BTC").unwrap();
        assert_eq!(
            format!("{:?}", currency),
            "Currency { id: \"BTC\", name: \"Bitcoin\", min_size: 0.00000001 }"
        );
        let currency = currencies.iter().find(|x| x.id == "USD").unwrap();
        assert_eq!(
            format!("{:?}", currency),
            "Currency { id: \"USD\", name: \"United States Dollar\", min_size: 0.01 }"
        );
    }

    #[test]
    #[cfg_attr(not(feature = "latency-tests"), ignore)] // region & opt-level dependent
    #[serial]
    fn test_check_latency() {
        delay();
        let client: Public<Sync> = Public::new(SANDBOX_URL);
        let _ = client.get_time().unwrap();
        let time = Instant::now();
        let _ = client.get_time().unwrap();
        let time = time.elapsed().subsec_millis();
        dbg!(time);
        assert!(time <= 500, "too slow")
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "latency-tests"), ignore)] // region & opt-level dependent
    #[serial]
    async fn test_check_latency_async_block_on() {
        delay();
        let client: Public<ASync> = Public::new(SANDBOX_URL);
        client.get_time().await.unwrap();
        let time = Instant::now();
        client.get_time().await.unwrap();
        let time = time.elapsed().subsec_millis();
        dbg!(time);
        assert!(time <= 150, "too slow")
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "latency-tests"), ignore)] // region & opt-level dependent
    #[serial]
    async fn test_check_latency_async() {
        delay();
        let client: Public<ASync> = Public::new(SANDBOX_URL);
        let _ = client.get_time().await.unwrap();
        let time = Instant::now();
        let _ = client.get_time().await.unwrap();
        let time = time.elapsed().subsec_millis();

        dbg!(time);
        assert!(time <= 150, "too slow")
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
