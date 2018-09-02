[![Build Status](https://travis-ci.org/inv2004/coinbase-pro-rs.svg?branch=master)](https://travis-ci.org/inv2004/coinbase-pro-rs)
[![Crates.io](https://img.shields.io/crates/v/coinbase-pro-rs.svg)](https://crates.io/crates/coinbase-pro-rs)
[![Docs.rs](https://docs.rs/coinbase-pro-rs/badge.svg)](https://docs.rs/coinbase-pro-rs)

# Coinbase pro client for Rust
Supports SYNC/ASYNC/Websocket-feed data support

## Example
Cargo.toml:
```toml
[dependencies]
coinbase-pro-rs = "0.2.0"
```

### Async
```rust
extern crate hyper;
extern crate tokio;
extern crate coinbase_pro_rs;

use hyper::rt::Future;
use coinbase_pro_rs::{Public, ASync, SANDBOX_URL};

fn main() {
    let client: Public<ASync> = Public::Public::new_with_keep_alive(SANDBOX_URL, false);
    // if keep_alive is not disables - tokio::run will hold the connection without exiting the example
    let f = client.get_time()
        .map_err(|_| ())
        .and_then(|time| {
            println!("Coinbase.time: {}", time.iso);
            Ok(())
        });

    tokio::run(f); // waiting for tokio
}
```
### Sync
```rust
extern crate coinbase_pro_rs;

use coinbase_pro_rs::{Public, Sync, SANDBOX_URL};

fn main() {
   let client: Public<Sync> = Public::new(SANDBOX_URL);
   let time = client.get_time().unwrap();
   println!("Coinbase.time: {}", time.iso);
}
```

## Api supported:
- [x] SYNC
- [x] ASYNC
- [x] Websocket-Feed

## API
- [x] Requests
- [ ] Pagination
- [x] Types
- [x] Private
  - [x] Authentication
  - [x] Accounts
  - [x] Orders
  - [x] Fills
  - [ ] Deposits
  - [ ] Withdrawals
  - [ ] Payment Methods
  - [ ] Coinbase Accounts
  - [ ] Reports
  - [x] User Account
- [x] Market Data
  - [x] Products
  - [x] Currencies
  - [x] Time
- [x] Websocket Feed
  - [x] heartbeat
  - [x] ticker
  - [x] level2
  - [ ] user
  - [x] matches
  - [x] full

## FIX API
by request

## OrderBook
after Websocket

### Tests
cargo test -- --test-threads=1
// to avoid "Rate limit exceeded" error
