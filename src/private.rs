use hyper::{Client, Request, Body, Uri};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use error::*;
use structs::*;
use super::Result;

trait ApiPrv {
    fn get_time(&self) -> Result<Time>;
}

pub struct Coinbase {
    uri: String,
    client: Client<HttpsConnector<HttpConnector>>,
    key: String,
    secret: String,
    passphrase: String
}

impl Coinbase {
    pub fn new(key: &str, secret: &str, passphrase: &str) -> Self {
        let https = HttpsConnector::new(4).unwrap();
        let client = Client::builder().build::<_, Body>(https);
        let uri = "https://api-public.sandbox.pro.coinbase.com".to_string();

        Self {
            uri,
            client,
            key: key.to_string(),
            secret: secret.to_string(),
            passphrase: passphrase.to_string()
        }
    }
}

