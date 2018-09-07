use structs::other::Error;

#[derive(Debug, Fail)]
pub enum CBError {
    #[fail(display = "http: {}", _0)]
    Http(#[cause] super::hyper::Error),
    #[fail(display = "serde: {}\n    {}", error, data)]
    Serde {
        #[cause]
        error: super::serde_json::Error,
        data: String,
    },
    #[fail(display = "coinbase: {}", _0)]
    Coinbase(Error),
    #[fail(display = "null")]
    Null,
}

#[derive(Debug, Fail)]
pub enum WSError {
    #[fail(display = "connect")]
    Connect(#[cause] super::tokio_tungstenite::tungstenite::Error),
    #[fail(display = "send")]
    Send(#[cause] super::tokio_tungstenite::tungstenite::Error),
    #[fail(display = "read")]
    Read(#[cause] super::tokio_tungstenite::tungstenite::Error),
    #[fail(display = "serde")]
    Serde {
        #[cause]
        error: super::serde_json::Error,
        data: String,
    },
}

use super::serde::{Deserialize, Deserializer};

impl<'de> Deserialize<'de> for WSError {
    fn deserialize<D>(_deserializer: D) -> Result<WSError, D::Error>
    where
        D: Deserializer<'de>,
    {
        unimplemented!()
    }
}
