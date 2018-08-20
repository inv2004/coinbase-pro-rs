extern crate serde;
extern crate serde_json;
extern crate tokio;

use hyper::{Client, Body};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;

use super::Result;
use structs::*;

pub trait ApiPub {
    fn get_time(&self) -> Result<Time>;
    fn get_currencies(&self) -> Result<Vec<Currency>>;
}

pub struct Coinbase {
    uri: String,
    client: Client<HttpsConnector<HttpConnector>>
}

impl super::Api for Coinbase {
    fn uri(&self) -> &str { &self.uri }
    fn client<'a>(&'a self) -> &'a Client<HttpsConnector<HttpConnector>> { &self.client }
}

impl Coinbase {
    pub fn new() -> Self {
        let https = HttpsConnector::new(4).unwrap();
        let client = Client::builder().build::<_, Body>(https);
        let uri = "https://api-public.sandbox.pro.coinbase.com".to_string();

        Self {
            uri,
            client
        }
    }
}

impl<T> ApiPub for super::Coinbase<T> where T: super::Api {
    fn get_time(&self) -> Result<Time> {
        self.get_sync("/time")
    }
    fn get_currencies(&self) -> Result<Vec<Currency>> {
        self.get_sync("/currencies")
    }
}

