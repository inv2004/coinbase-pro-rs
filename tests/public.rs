extern crate chrono;
extern crate coinbase_pro_rs;

mod common;

use chrono::prelude::*;
use coinbase_pro_rs::structs::public::*;
use coinbase_pro_rs::*;
use common::delay;

#[test]
fn test_get_time() {
    delay();
    let client: Public<Sync> = Public::new(SANDBOX_URL);
    let time = client.get_time().unwrap();
    let time_str = format!("{:?}", time);
    assert!(time_str.starts_with("Time {"));
    assert!(time_str.contains("iso:"));
    assert!(time_str.contains("epoch:"));
    assert!(time_str.ends_with("}"));
}

#[test]
fn test_get_products() {
    delay();
    let client: Public<Sync> = Public::new(SANDBOX_URL);
    let products = client.get_products().unwrap();
    let str = format!("{:?}", products);
    assert!(str.contains("{ id: \"BTC-USD\""));
}

#[test]
fn test_get_book() {
    delay();
    let client: Public<Sync> = Public::new(SANDBOX_URL);
    let book_l1 = client.get_book::<BookRecordL1>("BTC-USD").unwrap();
    let str1 = format!("{:?}", book_l1);
    assert_eq!(1, book_l1.bids.len());
    assert!(str1.contains("bids: [BookRecordL1 {"));
    let book_l2 = client.get_book::<BookRecordL2>("BTC-USD").unwrap();
    let str2 = format!("{:?}", book_l2);
    assert!(book_l2.bids.len() > 1);
    assert!(str2.contains("[BookRecordL2 {"));
    let book_l3 = client.get_book::<BookRecordL3>("BTC-USD").unwrap();
    let str3 = format!("{:?}", book_l3);
    assert!(book_l2.bids.len() > 1);
    assert!(str3.contains("[BookRecordL3 {"));
}

#[test]
fn test_get_ticker() {
    delay();
    let client: Public<Sync> = Public::new(SANDBOX_URL);
    let ticker = client.get_ticker("BTC-USD").unwrap();
    let str = format!("{:?}", ticker);
    assert!(str.starts_with("Ticker { trade_id:"));
    assert!(str.contains("time:"));
}

#[test]
fn test_get_trades() {
    delay();
    let client: Public<Sync> = Public::new(SANDBOX_URL);
    let trades = client.get_trades("BTC-USD").unwrap();
    assert!(trades.len() > 1);
    let str = format!("{:?}", trades);
    assert!(str.starts_with("[Trade { time: "));
}

#[test]
fn test_get_candles() {
    delay();
    let client: Public<Sync> = Public::new(SANDBOX_URL);
    let end = Utc::now();
    //        let start = end - Duration::minutes(10);
    let candles = client
        .get_candles("BTC-USD", None, Some(end), Granularity::M1)
        .unwrap();
    // let str = format!("{:?}", candles);
    // println!("{}", str);
    assert!(candles[0].0 > candles[1].0);
}

#[test]
fn test_get_stats24h() {
    delay();
    let client: Public<Sync> = Public::new(SANDBOX_URL);
    let stats24h = client.get_stats24h("BTC-USD").unwrap();
    let str = format!("{:?}", stats24h);
    assert!(str.contains("open:"));
    assert!(str.contains("high:"));
    assert!(str.contains("low:"));
    assert!(str.contains("volume:"));
}

#[test]
fn test_get_currencies() {
    delay();
    let client: Public<Sync> = Public::new(SANDBOX_URL);
    let currencies = client.get_currencies().unwrap();
    let currency = currencies.iter().find(|x| x.id == "BTC").unwrap();
    assert_eq!(
        format!("{:?}", currency),
        "Currency { id: \"BTC\", name: \"Bitcoin\", min_size: 0.00000001 }"
    );
    let currency = currencies.iter().find(|x| x.id == "LTC").unwrap();
    assert_eq!(
        format!("{:?}", currency),
        "Currency { id: \"LTC\", name: \"Litecoin\", min_size: 0.00000001 }"
    );
}

//    #[test]
//    fn test_tls() { // it hangs
//        let https = HttpsConnector::new(4).unwrap();
//        let client = Client::builder()
//            .build::<_, hyper::Body>(https);
//        let ft = client
//            .call("https://hyper.rs".parse().unwrap())
//            .map_err(|_| ())
//            .and_then(|res| {
//                res.into_body().concat2().map_err(|_| ())
//            })
//            .and_then(|body| {
//                println!("body: {:?}", &body);
//                Ok(())
//            });
//        rt::run(ft);
//    }
