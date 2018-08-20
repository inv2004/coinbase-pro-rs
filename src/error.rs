#[derive(Debug, Fail)]
pub enum CBError {
    #[fail(display = "http: {}", _0)]
    Http(#[cause] super::hyper::Error),
    #[fail(display = "serde: {}", _0)]
    Serde(#[cause] super::serde_json::Error),
    #[fail(display = "null")]
    Null
}


