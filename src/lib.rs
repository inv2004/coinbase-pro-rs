#[macro_use] extern crate serde_derive;
#[macro_use] extern crate failure;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate futures;

use std::fmt::Debug;
use hyper::{Client, Request, Body, Uri};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use hyper::rt::{Future, Stream};
use public::ApiPub;


pub mod error;
pub mod structs;
mod utils;
mod private;
mod public;

use structs::*;
use error::*;

pub type Result<T> = std::result::Result<T, CBError>;

trait Api {
    fn uri(&self) -> &str;
    fn client<'a>(&'a self) -> &'a Client<HttpsConnector<HttpConnector>>;
}
struct Coinbase<T>(T);

impl<T> Coinbase<T> where T: Api {
    fn get<U>(&self, uri: &str) -> impl Future<Item=U, Error=CBError>
        where for<'de> U: serde::Deserialize<'de>
    {
        let uri: Uri = (self.0.uri().to_string() + uri).parse().unwrap();
//        println!("{:?}", uri);

        let req = Request::get(uri )
            .header("User-Agent", "coinbase-pro-rs/0.1.0")
            .body(Body::empty())
            .unwrap();

        self.0.client()
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

    fn get_sync<U>(&self, uri: &str) -> Result<U>
        where U: Debug + Send + 'static,
              U: for<'de> serde::Deserialize<'de>
    {
        let mut rt = tokio::runtime::current_thread::Runtime::new().unwrap();
        rt.block_on(self.get(uri))
    }
}

impl Coinbase<public::Coinbase> {
    fn new() -> Self {
        Coinbase(public::Coinbase::new())
    }
}

impl Coinbase<private::Coinbase> {
    fn new() -> Self {
        Coinbase(private::Coinbase::new(KEY, SECRET, PASS))
    }
}

static KEY: &str = "c4f2ffd72b20836a0dc4ff0b2b658f72";
static PASS: &str = "testtesttest";
static SECRET: &str = "0bmte68VNnO3lHTfQdE4c+zfhruI10OIBXk8aq81NxdjAaz3C2Wo2t5xURxnNulcszQzjrCbY5HJjQv2d/bIXg==";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_time() {
        let b = Coinbase::<public::Coinbase>::new();
        let t = b.get_time().unwrap();
        assert!(format!("{:?}", t).starts_with("Time {"));
        assert!(format!("{:?}", t).contains("iso:"));
        assert!(format!("{:?}", t).contains("epoch:"));
        assert!(format!("{:?}", t).ends_with("}"));
    }

    #[test]
    fn test_get_currencies() {
        let b = Coinbase::<public::Coinbase>::new();
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

