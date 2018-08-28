[![Build Status](https://travis-ci.org/inv2004/coinbase-pro-rs.svg?branch=master)](https://travis-ci.org/inv2004/coinbase-pro-rs)
[![Crates.io](https://img.shields.io/crates/v/rustc-serialize.svg)](https://crates.io/crates/coinbase-pro-rs)
[![Docs.rs](https://docs.rs/coinbase-pro-rs/badge.svg)](https://docs.rs/coinbase-pro-rs)

# Coinbase pro client for Rust
Supports SYNC and ASYNC operations.

## Example
Cargo.toml:
```toml
[dependencies]
coinbase-pro-rs = "0.1.5"

```
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
- [ ] Websocket Feed
  - [ ] heartbeat
  - [ ] ticker
  - [ ] level2
  - [ ] user
  - [ ] matches
  - [ ] full

## FIX API
by request

## OrderBook
after Websocket

### test results
cargo test -- --test-threads=1
// to avoid "Rate limit exceeded" error

```
running 25 tests
test adapters::tests::test_async ... ok
test adapters::tests::test_sync ... ok
test private::tests::test_cancel_all ... ok
test private::tests::test_cancel_order ... ok
test private::tests::test_get_account ... ok
test private::tests::test_get_account_hist ... ok
test private::tests::test_get_account_holds ... ignored
test private::tests::test_get_accounts ... ok
test private::tests::test_get_fills ... ok
test private::tests::test_get_order ... ok
test private::tests::test_get_orders ... ignored
test private::tests::test_get_pub ... ok
test private::tests::test_get_trailing_volume ... ok
test private::tests::test_new_order_ser ... ok
test private::tests::test_set_order_limit ... ok
test private::tests::test_set_order_limit_gtc ... ok
test private::tests::test_set_order_market ... ok
test public::tests::test_get_book ... ok
test public::tests::test_get_candles ... ok
test public::tests::test_get_currencies ... ok
test public::tests::test_get_products ... ok
test public::tests::test_get_stats24h ... ok
test public::tests::test_get_ticker ... ok
test public::tests::test_get_time ... ok
test public::tests::test_get_trades ... ok

test result: ok. 23 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
```
