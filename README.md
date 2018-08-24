# Coinbase Pro client for Rust
Supports SYNC and ASYNC operations.

## Api supported:
* SYNC:  done
* ASYNC: NONE

## Methods
* Public:
* /time
* /products          NONE
* /currencies

Private:
* /accounts          GET
* /accounts/*
* /accounts/*/ledger
* /accounts/*/holds
* /orders            POST types: limit, market, stop, time_in_force, post
..* /orders/*          DELETE
* /orders            DELETE
* /orders            GET
* /orders/*          GET
* /fills             GET
* /deposits          NONE
* /withdrawals       NONE
* /payment-methods   NONE
* /coinbase-accounts NONE
* /reports           NONE
* /users/self/trailing-volume  NONE

## WebSocket:  NONE
* heartbeat
* ticker
* level2
* user
* matches
* full

## OrderBook
After websocket

### test results
```
running 15 tests
test private::tests::test_get_account ... ok
test private::tests::test_get_account_holds ... ignored
test private::tests::test_get_account_hist ... ok
test private::tests::test_cancel_order ... ok
test private::tests::test_get_accounts ... ok
test private::tests::test_get_orders ... ignored
test private::tests::test_new_order_ser ... ok
test private::tests::test_cancel_all ... ok
test private::tests::test_get_fills ... ok
test private::tests::test_get_order ... ok
test private::tests::test_set_order_limit_gtc ... ok
test public::tests::test_get_currencies ... ok
test private::tests::test_set_order_limit ... ok
test public::tests::test_get_time ... ok
test private::tests::test_set_order_market ... ok

test result: ok. 13 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
```