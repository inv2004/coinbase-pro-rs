#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate failure;
#[macro_use] extern crate log;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate tokio;
extern crate futures;
extern crate uuid;
extern crate pretty_env_logger;
extern crate chrono;

pub mod error;
pub mod structs;
mod utils;
mod private;
mod public;

use error::*;

pub type Result<T> = std::result::Result<T, CBError>;

