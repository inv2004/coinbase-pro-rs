#[macro_use] extern crate serde_derive;
#[macro_use] extern crate failure;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate futures;

pub mod error;
pub mod structs;
mod utils;
mod private;
mod public;

use error::*;

pub type Result<T> = std::result::Result<T, CBError>;

