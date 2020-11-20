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
//! use std::future::Future;
//! use coinbase_pro_rs::{Public, ASync, SANDBOX_URL};
//! use futures::{TryFutureExt};
//!
//! #[tokio::main]
//! async fn main() {
//!     let client: Public<ASync> = Public::new_with_keep_alive(SANDBOX_URL, false);
//!     // if keep_alive is not disables - tokio::run will hold the connection without exiting the example
//!     client.get_time().await
//!         .map_err(|_| ())
//!         .and_then(|time| {
//!             println!("Coinbase.time: {}", time.iso);
//!             Ok(())
//!         });
//!
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
//! use futures::{Future, Stream, StreamExt, TryStreamExt};
//! use coinbase_pro_rs::{WSFeed, CBError, WS_SANDBOX_URL};
//! use coinbase_pro_rs::structs::wsfeed::*;
//!
//! #[tokio::main]
//! async fn main() {
//!     let stream = WSFeed::new(WS_SANDBOX_URL,
//!         &["BTC-USD"], &[ChannelType::Heartbeat]);
//!
//!     stream
//!         .take(10)
//!         .for_each(|msg: Result<Message, CBError>| async {
//!         match msg.unwrap() {
//!             Message::Heartbeat {sequence, last_trade_id, time, ..} => println!("{}: seq:{} id{}",
//!                                                                                time, sequence, last_trade_id),
//!             Message::Error {message} => println!("Error: {}", message),
//!             Message::InternalError(_) => panic!("internal_error"),
//!             other => println!("{:?}", other)
//!         }
//!     }).await;
//! }
//! ```

pub mod adapters;
mod error;
pub mod private;
pub mod public;
pub mod structs;
mod utils;

pub mod wsfeed;

pub use crate::adapters::{ASync, Sync};
pub use crate::error::{CBError, WSError};
pub use crate::private::Private;
pub use crate::public::Public;
pub use crate::wsfeed::WSFeed;

pub type Result<T> = std::result::Result<T, CBError>;

/// https://api.pro.coinbase.com
pub const MAIN_URL: &str = "https://api.pro.coinbase.com";
/// https://api-public.sandbox.pro.coinbase.com
pub const SANDBOX_URL: &str = "https://api-public.sandbox.pro.coinbase.com";
/// wss://ws-feed.pro.coinbase.com
pub const WS_URL: &str = "wss://ws-feed.pro.coinbase.com";
/// wss://ws-feed-public.sandbox.pro.coinbase.com
pub const WS_SANDBOX_URL: &str = "wss://ws-feed-public.sandbox.pro.coinbase.com";
