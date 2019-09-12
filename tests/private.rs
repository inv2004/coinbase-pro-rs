extern crate coinbase_pro_rs;
extern crate serde_json;

mod common;

use coinbase_pro_rs::structs::reqs;
use coinbase_pro_rs::*;
use common::delay;
use coinbase_pro_rs::structs::reqs::{OrderTimeInForce, OrderTimeInForceCancelAfter};

static KEY: &str = "9eaa4603717ffdc322771a933ae12501";
static SECRET: &str =
    "RrLem7Ihmnn57ryW4Cc3Rp31h+Bm2DEPmzNbRiPrQQRE1yH6WNybmhK8xSqHjUNaR/V8huS+JMhBlr8PKt2GhQ==";
static PASSPHRASE: &str = "sandbox";

#[test]
fn test_get_accounts() {
    delay();
    let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
    let accounts = client.get_accounts().unwrap();
    assert!(
        format!("{:?}", accounts)
            .contains(r#"currency: "BTC""#)
    );
    assert!(
        format!("{:?}", accounts)
            .contains(r#"currency: "ETH""#)
    );
}

#[test]
fn test_get_account() {
    delay();
    //        super::super::pretty_env_logger::init_custom_env("RUST_LOG=trace");
    let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
    let coin_acc = client
        .get_accounts()
        .unwrap()
        .into_iter()
        .find(|x| x.currency == "BTC")
        .unwrap();
    let account = client.get_account(coin_acc.id);
    let account_str = format!("{:?}", account);
    assert!(account_str.contains("id:"));
    assert!(account_str.contains("currency: \"BTC\""));
    assert!(account_str.contains("balance:"));
    assert!(account_str.contains("available:"));
    assert!(account_str.contains("hold:"));
    assert!(account_str.contains("profile_id:"));
}

#[test]
fn test_get_account_hist() {
    delay();
    //        super::super::pretty_env_logger::init_custom_env("RUST_LOG=trace");
    let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
    let coin_acc = client
        .get_accounts()
        .unwrap()
        .into_iter()
        .find(|x| x.currency == "USD")
        .unwrap();
    let account = client.get_account_hist(coin_acc.id);
    let account_str = format!("{:?}", account);
    //        println!("{}", account_str);
    assert!(account_str.contains("type: Match, details: Match"));
}

#[test]
 #[ignore]
fn test_get_account_holds() {
    delay();
    //        super::super::pretty_env_logger::init_custom_env("RUST_LOG=trace");
    let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
    let coin_acc = client
        .get_accounts()
        .unwrap()
        .into_iter()
        .find(|x| x.currency == "USD")
        .unwrap();
    let acc_holds = client.get_account_holds(coin_acc.id);
    let _str = format!("{:?}", acc_holds);
    //        assert!(account_str.contains("transfer_type: Deposit"));
    //println!("{:?}", str);
    assert!(false); // TODO: holds are empty now
}

#[test]
fn test_new_order_ser() {
    delay();
    let order = reqs::Order::buy_market("BTC-UST", 1.1);
    let str = serde_json::to_string(&order).unwrap();
    assert_eq!(
        vec![0],
        str.match_indices("{").map(|(x, _)| x).collect::<Vec<_>>()
    );
}

#[test]
#[ignore] // sandbox price is too high
fn test_set_order_limit() {
    delay();
    let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
    let order = client.buy_limit("BTC-USD", 1.0, 1.12, true).unwrap();
    let str = format!("{:?}", order);
    assert!(str.contains("side: Buy"));
    assert!(str.contains("_type: Limit {"));
    let order = client
        .sell_limit("BTC-USD", 0.001, 100000.0, true)
        .unwrap();
    let str = format!("{:?}", order);
    assert!(str.contains("side: Sell"));
    assert!(str.contains("_type: Limit {"));
}

#[test]
fn test_set_order_limit_gtc() {
    delay();
    let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);

    let order = reqs::Order::buy_limit("BTC-USD", 1.0, 1.12, true)
        .time_in_force(OrderTimeInForce::GTT {cancel_after: OrderTimeInForceCancelAfter::Min});

    let order = client.set_order(order).unwrap();
    //        let order = client.buy("BTC-USD", 1.0).limit(1.0, 1.12).post_only().gtt(min).send()
    let str = format!("{:?}", order);
    assert!(str.contains("time_in_force: GTT { expire_time: 2"));
}

#[test]
fn test_set_order_stop() {
    delay();
    let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);

    let order = reqs::Order::buy_limit("BTC-USD", 1.0, 1.12, false)
        .stop_entry(0.8)
        .time_in_force(OrderTimeInForce::GTT {cancel_after: OrderTimeInForceCancelAfter::Min});

    let str = serde_json::to_string(&order).unwrap();
    assert!(str.contains("stop_price\":0.8,\"stop\":\"entry\""));

    let order = client.set_order(order).unwrap();
    assert!(order.stop.is_none());
}

#[test]
#[ignore] // sandbox price is too high
fn test_set_order_market() {
    delay();
    let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
    let order = client.buy_market("BTC-USD", 0.001).unwrap();
    let str = format!("{:?}", order);
    assert!(str.contains("side: Buy"));
    assert!(str.contains("_type: Market {"));
    let order = client.sell_market("BTC-USD", 0.001).unwrap();
    let str = format!("{:?}", order);
    assert!(str.contains("side: Sell"));
    assert!(str.contains("_type: Market {"));
    assert!(false);
}

#[test]
fn test_cancel_order() {
    delay();
    let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
    let order = client.buy_limit("BTC-USD", 1.0, 1.12, true).unwrap();
    let res = client.cancel_order(order.id).unwrap();
    assert_eq!(order.id, res);
}

#[test]
fn test_cancel_all() {
    delay();
    let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
    let order1 = client.buy_limit("BTC-USD", 1.0, 1.12, true).unwrap();
    let order2 = client.buy_limit("BTC-USD", 1.0, 1.12, true).unwrap();
    let res = client.cancel_all(Some("BTC-USD")).unwrap();
    assert!(res.iter().find(|x| **x == order1.id).is_some());
    assert!(res.iter().find(|x| **x == order2.id).is_some());
}

#[test]
#[ignore]
fn test_get_orders() {
    delay();
    let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
    let orders = client.get_orders(None, None).unwrap();
    let str = format!("{:?}", orders);
    println!("{}", str);
    assert!(false);
}

#[test]
fn test_get_order() {
    delay();
    let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
    let order = client.buy_limit("BTC-USD", 1.0, 1.12, true).unwrap();
    let order_res = client.get_order(order.id).unwrap();
    assert_eq!(order.id, order_res.id);
}

#[test]
fn test_get_fills() {
    delay();
    let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
    let fills = client.get_fills(None, Some("BTC-USD")).unwrap();
    if fills.len() > 0 {
        let str = format!("{:?}", fills);
        assert!(str.contains("Fill { trade_id: "));
    }
}

#[test]
#[ignore]
fn test_get_trailing_volume() {
    delay();
    let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
    let vols = client.get_trailing_volume().unwrap();
    let str = format!("{:?}", vols);
    assert!(str == "[]"); // nothing now
}

#[test]
fn test_get_pub() {
    delay();
    let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
    let time = client.public().get_time().unwrap();
    let time_str = format!("{:?}", time);
    assert!(time_str.starts_with("Time {"));
    assert!(time_str.contains("iso:"));
    assert!(time_str.contains("epoch:"));
    assert!(time_str.ends_with("}"));
}
