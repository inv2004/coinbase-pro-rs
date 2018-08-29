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
    Read(#[cause] super::tokio_tungstenite::tungstenite::Error)
}