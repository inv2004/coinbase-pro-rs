extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate base64;
extern crate hmac;
extern crate sha2;

use std::fmt::Debug;
use hyper::{HeaderMap};
use hyper::header::HeaderValue;
use private::hmac::{Hmac, Mac};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use super::Result;
use structs::*;

use public::Public;

pub struct Private {
    _pub: Public,
    key: String,
    secret: String,
    passphrase: String
}

impl Private {
    pub fn get_sync<U>(&self, uri: &str) -> Result<U>
        where U: Debug + Send + 'static,
              U: for<'de> serde::Deserialize<'de>
    {
        self._pub.get_sync_with_headers(uri, self.headers(uri))
    }

    fn sign(&self, timestamp: u64, uri: &str) -> String {
        let key = base64::decode(&self.secret).expect("base64::decode secret");
        let mut mac: Hmac<sha2::Sha256> = Hmac::new_varkey(&key).expect("Hmac::new(key)");
        mac.input((timestamp.to_string()+"GET"+uri+"").as_bytes());
        base64::encode(&mac.result().code())
    }

    fn headers(&self, uri: &str) -> HeaderMap {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("leap-second").as_secs();

        let mut headers = HeaderMap::new();
        headers.insert("CB-ACCESS-KEY", HeaderValue::from_str(&self.key).unwrap());
        headers.insert("CB-ACCESS-SIGN", HeaderValue::from_str(&self.sign(timestamp, uri)).unwrap());
        headers.insert("CB-ACCESS-TIMESTAMP", HeaderValue::from_str(&timestamp.to_string()).unwrap());
        headers.insert("CB-ACCESS-PASSPHRASE", HeaderValue::from_str(&self.passphrase).unwrap());
        trace!("{:?}", headers);
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
        self.get_sync("/accounts")
    }

    pub fn get_account(&self, id: Uuid) -> Result<Account> {
        self.get_sync(&format!("/accounts/{}", id))
    }

    pub fn get_account_hist(&self, id: Uuid) -> Result<Vec<AccountHistory>> {
        self.get_sync(&format!("/accounts/{}/ledger", id))
            .map(|xs: Vec<AccountHistory>| xs.into_iter()
                 .map(|x| {
                     let _type = x.details.clone().into(); //clone???
                     AccountHistory{_type, ..x}
                 }).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static KEY: &str = "1d0dc0f7b4e808d430b95d8fed7df3ea";
    static PASS_PHRASE: &str = "sandbox";
    static SECRET: &str = "dTUic8DZPqkS77vxhJFEX5IBr13FcFHTzWYOARgT9kDWGdN03uvxBbH/hVy8f4O5RDmuf+9wNpEfhYhw2FCWyA==";

    #[test]
    fn test_get_accounts() {
        let client = Private::new(KEY, SECRET, PASS_PHRASE);
        let accounts = client.get_accounts().unwrap();
        assert!(format!("{:?}", accounts)
            .contains(r#"currency: "BCH", balance: 0.0, available: 0.0, hold: 0.0, profile_id: "#));
        assert!(format!("{:?}", accounts)
            .contains(r#"currency: "ETH", balance: 0.0, available: 0.0, hold: 0.0, profile_id: "#));
    }

    #[test]
    fn test_get_account() {
//        super::super::pretty_env_logger::init_custom_env("RUST_LOG=trace");
        let client = Private::new(KEY, SECRET, PASS_PHRASE);
        let coin = client.get_accounts().unwrap().into_iter().find(|x| x.currency == "BTC").unwrap();
        let account = client.get_account(coin.id);
        let account_str = format!("{:?}", account);
        assert!(account_str.contains("id:"));
        assert!(account_str.contains("currency: \"BTC\""));
        assert!(account_str.contains("balance:"));
        assert!(account_str.contains("available:"));
        assert!(account_str.contains("hold:"));
        assert!(account_str.contains("profile_id:"));
    }

    #[test]
    fn test_get_account_hist() {
//        super::super::pretty_env_logger::init_custom_env("RUST_LOG=trace");
        let client = Private::new(KEY, SECRET, PASS_PHRASE);
        let coin = client.get_accounts().unwrap().into_iter().find(|x| x.currency == "USD").unwrap();
        let account = client.get_account_hist(coin.id);
        let account_str = format!("{:?}", account);
        assert!(account_str.contains("transfer_type: Deposit"));
    }
}

