extern crate tokio;
extern crate serde_json;
extern crate coinbase_pro_rs;

use tokio::prelude::{Future, Stream};
use coinbase_pro_rs::{WSFeed, wsfeed::WS_SANDBOX_URL};
use coinbase_pro_rs::structs::wsfeed::*;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

#[test]
fn test_subscribe() {
    let s = Subscribe {
        _type: SubscribeCmd::Subscribe,
        product_ids: vec!["BTC-USD".to_string()],
        channels: vec![Channel::Name(ChannelType::Heartbeat),
                       Channel::WithProduct {
                           name: ChannelType::Level2,
                           product_ids: vec!["BTC-USD".to_string()]
                       }]
    };

    let str = serde_json::to_string(&s).unwrap();
    assert_eq!(str,
               r#"{"type":"subscribe","product_ids":["BTC-USD"],"channels":["heartbeat",{"name":"level2","product_ids":["BTC-USD"]}]}"#);
}

#[test]
fn test_subscription() {
    let stream = WSFeed::new(WS_SANDBOX_URL,
                             &["BTC-USD"], &[ChannelType::Heartbeat]);
    let f = stream
        .take(1)
        .for_each(move |msg| {
            let str = format!("{:?}", msg);
            assert_eq!(str, r#"Subscriptions { channels: [WithProduct { name: Heartbeat, product_ids: ["BTC-USD"] }] }"#);
            Ok(())
        });

    tokio::runtime::run(f.map_err(|e| println!("{:?}", e)));
}

#[test]
fn test_heartbeat() {
    let found = Arc::new(AtomicBool::new(false));
    let found2 = found.clone();
    let stream = WSFeed::new(WS_SANDBOX_URL,
                             &["BTC-USD"], &[ChannelType::Heartbeat]);
    let f = stream
        .take(3)
        .for_each(move |msg| {
            let str = format!("{:?}", msg);
            if str.starts_with("Heartbeat { sequence: ") {
                found2.swap(true, Ordering::Relaxed);
            }
            Ok(())
        });

    tokio::runtime::run(f.map_err(|e| println!("{:?}", e)));

    assert!(found.load(Ordering::Relaxed));
}

#[test]
#[ignore] // hard to check in sandbox because flow is very low
fn test_ticker() {
    let found = Arc::new(AtomicBool::new(false));
    let found2 = found.clone();

    let pairs = vec!["BTC-USD", "BTC-EUR", "BTC-GBP", "ETH-USD", "ETH-EUR", "LTC-USD", "LTC-EUR"];

    let stream = WSFeed::new(WS_SANDBOX_URL,
                             &pairs, &[ChannelType::Ticker]);
    let f = stream
        .take(10    )
        .for_each(move |msg| {
            let str = format!("{:?}", msg);
            if str.contains("Ticker(Full { trade_id: ") {
                found2.swap(true, Ordering::Relaxed);
            }
            Ok(())
        });

    tokio::runtime::run(f.map_err(|e| println!("{:?}", e)));

    assert!(found.load(Ordering::Relaxed));
}

//#[test]
//fn test_level2() {
//    let found = Arc::new(AtomicBool::new(false));
//    let found2 = found.clone();
//
//    let pairs = vec!["BTC-USD", "BTC-EUR", "BTC-GBP", "ETH-USD", "ETH-EUR", "LTC-USD", "LTC-EUR"];
//
//    let stream = WSFeed::new(WS_SANDBOX_URL,
//                             &pairs, &[ChannelType::Level2]);
//    let f = stream
//        .take(3    )
//        .for_each(move |msg| {
//            let str = format!("{:?}", msg);
//            println!("{}", str);
//            Ok(())
//        });
//
//    tokio::runtime::run(f.map_err(|e| println!("{:?}", e)));
//
//    assert!(found.load(Ordering::Relaxed));
//}

