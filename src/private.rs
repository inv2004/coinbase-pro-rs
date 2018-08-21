extern crate serde;
extern crate serde_json;
extern crate tokio;

use std::fmt::Debug;
use hyper::{Client, Request, Body, Uri};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use hyper::rt::{Future, Stream};

use super::Result;
use error::*;
use structs::*;

use public::Public;

pub struct Private {
    _pub: Public,
    key: String,
    secret: String,
    passphrase: String
}

impl Private {
    pub fn new(key: &str, secret: &str, passphrase: &str) -> Self {
        Self {
            _pub: Public::new(),
            key: key.to_string(),
            secret: secret.to_string(),
            passphrase: passphrase.to_string()
        }
    }

    pub fn get_accounts(&self) -> Result<Vec<Account>> {
        self._pub.get_sync("/accounts")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static KEY: &str = "c4f2ffd72b20836a0dc4ff0b2b658f72";
    static PASS: &str = "testtesttest";
    static SECRET: &str = "0bmte68VNnO3lHTfQdE4c+zfhruI10OIBXk8aq81NxdjAaz3C2Wo2t5xURxnNulcszQzjrCbY5HJjQv2d/bIXg==";

    #[test]
    fn test_get_accounts() {
        let b = Private::new(KEY, PASS, SECRET);
        let t = b.get_accounts().unwrap();
    }
}

