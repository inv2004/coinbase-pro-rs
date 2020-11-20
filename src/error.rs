// use crate::structs::other::Error;
#![forbid(missing_docs)]
use thiserror::Error;

/// Coinbase-pro-rs error
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CBError {
    /// Http error
    #[error("http: {0}")]
    Http(#[source] hyper::Error),

    /// Serde error
    #[error("serde: {error}\n    {data}")]
    Serde {
        /// Underlying json error
        #[source]
        error: serde_json::Error,

        /// Data associated with serde error
        data: String,
    },

    /// Coinbase Error
    #[error("coinbase: {0}")]
    Coinbase(String),

    /// Websocket error
    #[error("websocket: {0}")]
    Websocket(WSError),

    /// Null error
    #[error("null")]
    Null,
}

/// Websocket specific errors
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum WSError {
    /// Error making Websocket connection
    #[error("connect")]
    Connect(#[source] tokio_tungstenite::tungstenite::Error),

    /// Error sending websocket message
    #[error("send")]
    Send(#[source] tokio_tungstenite::tungstenite::Error),

    /// Error reading from websocket
    #[error("read")]
    Read(#[source] tokio_tungstenite::tungstenite::Error),
}

use serde::{Deserialize, Deserializer};

impl<'de> Deserialize<'de> for WSError {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        unimplemented!()
    }
}

impl<'de> Deserialize<'de> for CBError {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        unimplemented!()
    }
}
