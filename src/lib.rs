//! Coinbase pro client with sync/async + websocket-feed support
//!
//! ## Structure
//!
//! There are two main structures to work with: [`Private`] and [`Public`], which provide interfaces to
//! work with [https://docs.pro.coinbase.com](https://docs.pro.coinbase.com) .
//! The structures should be parametrised with: [`Sync`] or [`ASync`] adapter-type, which blocks
//! future and returns result of its execution for Sync adapter or returns Future for ASync
//! adapter.
//!
//! [`WSFeed`] provides futures::Stream of websocket message for different channels.
//!
//! ## Examples
//!
//! ### Async
//! ```
//! extern crate hyper;
//! extern crate tokio;
//! extern crate coinbase_pro_rs;
//!
//! use hyper::rt::Future;
//! use coinbase_pro_rs::{Public, ASync, SANDBOX_URL};
//!
//! fn main() {
//!     let client: Public<ASync> = Public::new_with_keep_alive(SANDBOX_URL, false);
//!     // if keep_alive is not disables - tokio::run will hold the connection without exiting the example
//!     let f = client.get_time()
//!         .map_err(|_| ())
//!         .and_then(|time| {
//!             println!("Coinbase.time: {}", time.iso);
//!             Ok(())
//!         });
//!
//!     tokio::run(f);
//! }
//! ```
//! ### Sync
//! ```
//! extern crate coinbase_pro_rs;
//!
//! use coinbase_pro_rs::{Public, Sync, SANDBOX_URL};
//!
//! fn main() {
//!    let client: Public<Sync> = Public::new(SANDBOX_URL);
//!    let time = client.get_time().unwrap();
//!    println!("Coinbase.time: {}", time.iso);
//!}
//! ```
//! ### Websocket
//! ```
//! extern crate futures;
//! extern crate tokio;
//! extern crate coinbase_pro_rs;
//!
//! use futures::{Future, Stream};
//! use coinbase_pro_rs::{WSFeed, WS_SANDBOX_URL};
//! use coinbase_pro_rs::structs::wsfeed::*;
//!
//! fn main() {
//!     let stream = WSFeed::new(WS_SANDBOX_URL,
//!         &["BTC-USD"], &[ChannelType::Heartbeat]);
//!
//!     let f = stream
//!         .take(10)
//!         .for_each(|msg| {
//!         match msg {
//!             Message::Heartbeat {sequence, last_trade_id, time, ..} => println!("{}: seq:{} id{}",
//!                                                                                time, sequence, last_trade_id),
//!             Message::Error {message} => println!("Error: {}", message),
//!             Message::InternalError(_) => panic!("internal_error"),
//!             other => println!("{:?}", other)
//!         }
//!         Ok(())
//!     });
//!
//!     tokio::run(f.map_err(|_| panic!("stream fail")));
//! }
//! ```

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate serde_json;
extern crate chrono;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate pretty_env_logger;
extern crate serde;
extern crate time;
extern crate tokio;
extern crate uuid;
extern crate tokio_tungstenite;

pub mod error;
pub mod private;
pub mod public;
pub mod structs;
pub mod adapters;
mod utils;

pub mod wsfeed;

pub use private::Private;
pub use public::Public;
pub use wsfeed::WSFeed;
pub use error::CBError;
pub use error::WSError;
pub use adapters::{Sync, ASync};

pub type Result<T> = std::result::Result<T, CBError>;

/// https://api.pro.coinbase.com
pub const MAIN_URL: &str = "https://api.pro.coinbase.com";
/// https://api-public.sandbox.pro.coinbase.com
pub const SANDBOX_URL: &str = "https://api-public.sandbox.pro.coinbase.com";
/// wss://ws-feed.pro.coinbase.com
pub const WS_URL: &str = "wss://ws-feed.pro.coinbase.com";
/// wss://ws-feed-public.sandbox.pro.coinbase.com
pub const WS_SANDBOX_URL: &str = "wss://ws-feed-public.sandbox.pro.coinbase.com";



