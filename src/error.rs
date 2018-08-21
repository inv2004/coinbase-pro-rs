use structs::Error;

#[derive(Debug, Fail)]
pub enum CBError {
    #[fail(display = "http: {}", _0)]
    Http(#[cause] super::hyper::Error),
    #[fail(display = "serde: {}\n    {}", error, data)]
    Serde {
        #[cause] error: super::serde_json::Error,
        data: String
    },
    #[fail(display = "coinbase: {}", _0)]
    Coinbase (Error),
    #[fail(display = "null")]
    Null
}


