#[macro_use] extern crate serde_derive;
#[macro_use] extern crate failure;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate futures;

pub mod error;
pub mod structs;
pub mod public;
mod utils;

use error::*;
use structs::*;
use public::*;
use hyper::{Client, Request};
use hyper::rt::{Future, Stream};
use hyper_tls::HttpsConnector;

struct Coinbase {
    uri: String,
    client: hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>
}

impl Coinbase {
    fn new() -> Self {
        let https = HttpsConnector::new(4).unwrap();
        let client = Client::builder().build::<_, hyper::Body>(https);
//        let uri = "https://api.gdax.com".to_string();
        let uri = "https://api-public.sandbox.pro.coinbase.com".to_string();

        Self {
            uri,
            client
        }
    }

    fn get<T>(&self, uri: &str) -> impl Future<Item=T, Error=CBError>
        where for<'de> T: serde::Deserialize<'de>
    {
        let uri: hyper::Uri = (self.uri.to_string() + uri).parse().unwrap();
//        println!("{:?}", uri);

        let req = Request::get(uri)
            .header("User-Agent", "coinbase-pro-rs/0.1.0")
            .body(hyper::Body::empty())
            .unwrap();

        self.client
            .request(req)
            .map_err(CBError::Http)
            .and_then(|res| {
                res.into_body().concat2().map_err(CBError::Http)
            })
            .and_then(|body| {
                let res = serde_json::from_slice(&body).map_err(CBError::Serde)?;
                Ok(res)
            })
    }

    fn get_sync<T>(&self, uri: &str) -> Result<T>
        where T: std::fmt::Debug + Send + 'static,
              T: for<'de> serde::Deserialize<'de>
    {
        let mut rt = tokio::runtime::current_thread::Runtime::new().unwrap();
        rt.block_on(self.get(uri))
    }

}

impl Public for Coinbase {
    fn get_time(&self) -> Result<Time> {
        self.get_sync("/time")
    }
    fn get_currencies(&self) -> Result<Vec<Currency>> {
        self.get_sync("/currencies")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_time() {
        let b = Coinbase::new();
        let t = b.get_time().unwrap();
        assert!(format!("{:?}", t).starts_with("Time {"));
        assert!(format!("{:?}", t).contains("iso:"));
        assert!(format!("{:?}", t).contains("epoch:"));
        assert!(format!("{:?}", t).ends_with("}"));
    }

    #[test]
    fn test_get_currencies() {
        let b = Coinbase::new();
        let cs = b.get_currencies().unwrap();
        let c = cs.iter().find(|x| x.id == "BTC").unwrap();
        assert_eq!(format!("{:?}", c), "Currency { id: \"BTC\", name: \"Bitcoin\", min_size: 0.00000001 }");
        let c = cs.iter().find(|x| x.id == "LTC").unwrap();
        assert_eq!(format!("{:?}", c), "Currency { id: \"LTC\", name: \"Litecoin\", min_size: 0.00000001 }");
    }

//    #[test]
//    fn test_tls() {
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

