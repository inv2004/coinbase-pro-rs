//! Contains structure which provides access to Public section of Coinbase api

extern crate serde;
extern crate serde_json;
extern crate tokio;

use hyper::client::HttpConnector;
use hyper::rt::{Future, Stream};
use hyper::{Body, Client, Request, Uri};
use hyper_tls::HttpsConnector;
use serde::Deserialize;
use std::fmt::Debug;
use std::marker::PhantomData;

use super::adapters::*;
use error::*;
use structs::public::*;
use structs::DateTime;

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

        let mut req = Request::get(uri);
        req.header("User-Agent", Self::USER_AGENT);
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
    ) -> impl Future<Item = U, Error = CBError>
    where
        for<'de> U: serde::Deserialize<'de>,
    {
        debug!("REQ: {:?}", request);

        self.client
            .request(request)
            .map_err(CBError::Http)
            .and_then(|res| res.into_body().concat2().map_err(CBError::Http))
            .and_then(|body| {
                debug!("RES: {:?}", body);
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
        A: AdapterNew
    {
        let https = HttpsConnector::new(4).unwrap();
        let client = Client::builder()
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
        A: AdapterNew
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

    pub fn get_book<T>(&self, product_id: &str) -> A::Result
    where
        A: Adapter<Book<T>> + 'static,
        T: BookLevel + Debug + 'static,
        T: super::std::marker::Send,
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
            .map(|x| format!("&start={}", x.to_rfc3339()))
            .unwrap_or_default();
        let param_end = end
            .map(|x| format!("&end={}", x.to_rfc3339()))
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
