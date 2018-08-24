extern crate serde;
extern crate serde_json;
extern crate tokio;

use std::fmt::Debug;
use hyper::{Client, Request, Body, Uri, HeaderMap};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use hyper::rt::{Future, Stream};
use serde::Deserialize;

use super::Result;
use error::*;
use structs::public::*;

pub struct Public {
    pub uri: String,
    client: Client<HttpsConnector<HttpConnector>>,
}

impl Public {
    pub const USER_AGENT: &'static str = "coinbase-pro-rs/0.1.0";

    fn request(&self, uri: &str) -> Request<Body> {
        let uri: Uri = (self.uri.to_string() + uri).parse().unwrap();

        let mut req = Request::get(uri);
        req.header("User-Agent", Self::USER_AGENT);
        req.body(Body::empty()).unwrap()
    }

    pub fn get<U>(&self, request: Request<Body>) -> impl Future<Item=U, Error=CBError>
        where for<'de> U: serde::Deserialize<'de>
    {
        debug!("{:?}", request);

        self.client
            .request(request)
            .map_err(CBError::Http)
            .and_then(|res| {
                res.into_body().concat2().map_err(CBError::Http)
            })
            .and_then(|body| {
                let res = serde_json::from_slice(&body)
                    .map_err(|e| {
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

    pub fn get_sync_with_req<U>(&self, request: Request<Body>) -> Result<U>
        where U: Debug + Send + 'static,
              U: for<'de> serde::Deserialize<'de>
    {
        let mut rt = tokio::runtime::current_thread::Runtime::new().unwrap();
        rt.block_on(self.get(request))
    }

    pub fn get_sync<U>(&self, uri: &str) -> Result<U>
        where U: Debug + Send + 'static,
              U: for<'de> serde::Deserialize<'de>
    {
        self.get_sync_with_req(self.request(uri))
    }

    pub fn new() -> Self {
        let https = HttpsConnector::new(4).unwrap();
        let client = Client::builder().build::<_, Body>(https);
        let uri = "https://api-public.sandbox.pro.coinbase.com".to_string();

        Self {
            uri,
            client
        }
    }

    pub fn get_time(&self) -> Result<Time> {
        self.get_sync("/time")
    }

    pub fn get_products(&self) -> Result<Vec<Product>> {
        self.get_sync("/products")
    }

    pub fn get_book<T>(&self, product_id: &str) -> Result<Book<T>>
        where T: BookLevel + Debug + 'static,
              T: super::std::marker::Send,
              T: for<'de> Deserialize<'de>
    {
        self.get_sync(&format!("/products/{}/book?level={}", product_id, T::level()))
    }

    pub fn get_currencies(&self) -> Result<Vec<Currency>> {
        self.get_sync("/currencies")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_time() {
        let client = Public::new();
        let time = client.get_time().unwrap();
        let time_str = format!("{:?}", time);
        assert!(time_str.starts_with("Time {"));
        assert!(time_str.contains("iso:"));
        assert!(time_str.contains("epoch:"));
        assert!(time_str.ends_with("}"));
    }

    #[test]
    fn test_get_currencies() {
        let client = Public::new();
        let currencies = client.get_currencies().unwrap();
        let currency = currencies.iter().find(|x| x.id == "BTC").unwrap();
        assert_eq!(format!("{:?}", currency), "Currency { id: \"BTC\", name: \"Bitcoin\", min_size: 0.00000001 }");
        let currency = currencies.iter().find(|x| x.id == "LTC").unwrap();
        assert_eq!(format!("{:?}", currency), "Currency { id: \"LTC\", name: \"Litecoin\", min_size: 0.00000001 }");
    }

//    #[test]
//    fn test_tls() { // it hangs
//        let https = HttpsConnector::new(4).unwrap();
//        let client = Client::builder()
//            .build::<_, hyper::Body>(https);
//        let ft = client
//            .get("https://hyper.rs".parse().unwrap())
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

