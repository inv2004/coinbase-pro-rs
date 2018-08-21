extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate base64;
extern crate hmac;
extern crate sha2;

use std::fmt::Debug;
use hyper::{Client, Request, Body, Uri, HeaderMap};
use hyper::client::HttpConnector;
use hyper::header::HeaderValue;
use hyper_tls::HttpsConnector;
use hyper::rt::{Future, Stream};
use private::hmac::{Hmac, Mac};
use std::time::{SystemTime, UNIX_EPOCH};

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
    fn sign(&self, timestamp: u64, uri: &str) -> String {
        let key = base64::decode(&self.secret).expect("base64::decode secret");
        let mut mac: Hmac<sha2::Sha256> = Hmac::new_varkey(&key).expect("Hmac::new(key)");
        mac.input(format!("{}{}{}{}", timestamp, "GET", uri, "").as_bytes());
        base64::encode(&mac.result().code())
    }

    fn headers(&self, uri: &str) -> HeaderMap {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("leap-second").as_secs();

        let mut headers = HeaderMap::new();
        headers.insert("CB-ACCESS-KEY", HeaderValue::from_str(&self.key).unwrap());
        headers.insert("CB-ACCESS-SIGN", HeaderValue::from_str(&self.sign(timestamp, uri)).unwrap());
        headers.insert("CB-ACCESS-TIMESTAMP", HeaderValue::from_str(&timestamp.to_string()).unwrap());
        headers.insert("CB-ACCESS-PASSPHRASE", HeaderValue::from_str(&self.passphrase).unwrap());
        headers
    }

    pub fn new(key: &str, secret: &str, passphrase: &str) -> Self {
        Self {
            _pub: Public::new(),
            key: key.to_string(),
            secret: secret.to_string(),
            passphrase: passphrase.to_string()
        }
    }

    pub fn get_accounts(&self) -> Result<Vec<Account>> {
        self._pub.get_sync("/accounts", self.headers("/accounts"))
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

