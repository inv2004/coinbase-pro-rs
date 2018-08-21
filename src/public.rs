extern crate serde;
extern crate serde_json;
extern crate tokio;

use std::fmt::Debug;
use hyper::{Client, Request, Body, Uri, HeaderMap};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use hyper::rt::{Future, Stream};
use hyper::header::HeaderValue;
use failure::Fail;

use super::Result;
use error::*;
use structs::*;

pub struct Public {
    uri: String,
    client: Client<HttpsConnector<HttpConnector>>
}

impl Public {
    fn get_headers(&self) -> HeaderMap {
        HeaderMap::new()
    }

    pub fn get<U>(&self, uri: &str, headers: HeaderMap) -> impl Future<Item=U, Error=CBError>
        where for<'de> U: serde::Deserialize<'de>
    {
        let uri: Uri = (self.uri.to_string() + uri).parse().unwrap();
//        println!("{:?}", uri);

        let mut req = Request::get(uri);
        req.header("User-Agent", "coinbase-pro-rs/0.1.0");

        headers.iter().for_each(|(k,v)| {
            req.header(k, v);
        });

        self.client
            .request(req.body(Body::empty()).unwrap())
            .map_err(CBError::Http)
            .and_then(|res| {
                res.into_body().concat2().map_err(CBError::Http)
            })
            .and_then(|body| {
                let res = serde_json::from_slice(&body)
                    .map_err(|e| {
                        let data = String::from_utf8(body.to_vec()).unwrap();
                        CBError::Serde{error: e, data}
                    })?;
                Ok(res)
            })
    }

    pub fn get_sync<U>(&self, uri: &str, headers: HeaderMap) -> Result<U>
        where U: Debug + Send + 'static,
              U: for<'de> serde::Deserialize<'de>
    {
        let mut rt = tokio::runtime::current_thread::Runtime::new().unwrap();
        rt.block_on(self.get(uri, headers))
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
        self.get_sync("/time", self.get_headers())
    }
    pub fn get_currencies(&self) -> Result<Vec<Currency>> {
        self.get_sync("/currencies", self.get_headers())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_time() {
        let b = Public::new();
        let t = b.get_time().unwrap();
        assert!(format!("{:?}", t).starts_with("Time {"));
        assert!(format!("{:?}", t).contains("iso:"));
        assert!(format!("{:?}", t).contains("epoch:"));
        assert!(format!("{:?}", t).ends_with("}"));
    }

    #[test]
    fn test_get_currencies() {
        let b = Public::new();
        let cs = b.get_currencies().unwrap();
        let c = cs.iter().find(|x| x.id == "BTC").unwrap();
        assert_eq!(format!("{:?}", c), "Currency { id: \"BTC\", name: \"Bitcoin\", min_size: 0.00000001 }");
        let c = cs.iter().find(|x| x.id == "LTC").unwrap();
        assert_eq!(format!("{:?}", c), "Currency { id: \"LTC\", name: \"Litecoin\", min_size: 0.00000001 }");
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

