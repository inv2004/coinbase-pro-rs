//! Coinbase pro client with sync and async support
//!
//! ## Example
//!
//! ```
//! extern crate coinbase_pro_rs;
//!
//! use coinbase_pro_rs::{Public, Sync};
//!
//! fn main() {
//!    let client: Public<Sync> = Public::new();
//!    let time = client.get_time().unwrap();
//!    println!("Coinbase.time: {}", time.iso);
//!}
//! ```

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate chrono;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate pretty_env_logger;
extern crate serde;
extern crate time;
extern crate tokio;
extern crate uuid;

pub mod error;
pub mod private;
pub mod public;
pub mod structs;
pub mod adapters;
mod utils;

pub use private::Private;
pub use public::Public;
pub use error::CBError;
pub use adapters::{Sync, ASync};

pub type Result<T> = std::result::Result<T, CBError>;

