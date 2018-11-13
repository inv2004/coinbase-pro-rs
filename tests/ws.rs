extern crate coinbase_pro_rs;
extern crate serde_json;
extern crate tokio;

mod common;

use coinbase_pro_rs::structs::wsfeed::*;
use coinbase_pro_rs::{WSFeed, WS_SANDBOX_URL, WS_URL};
use common::delay;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::prelude::{Future, Stream};

static KEY: &str = "9eaa4603717ffdc322771a933ae12501";
static SECRET: &str =
    "RrLem7Ihmnn57ryW4Cc3Rp31h+Bm2DEPmzNbRiPrQQRE1yH6WNybmhK8xSqHjUNaR/V8huS+JMhBlr8PKt2GhQ==";
static PASSPHRASE: &str = "sandbox";

#[test]
fn test_subscribe() {
    delay();
    let s = Subscribe {
        _type: SubscribeCmd::Subscribe,
        product_ids: vec!["BTC-USD".to_string()],
        channels: vec![
            Channel::Name(ChannelType::Heartbeat),
            Channel::WithProduct {
                name: ChannelType::Level2,
                product_ids: vec!["BTC-USD".to_string()],
            },
        ],
        auth: None
    };

    let str = serde_json::to_string(&s).unwrap();
    assert_eq!(str,
               r#"{"type":"subscribe","product_ids":["BTC-USD"],"channels":["heartbeat",{"name":"level2","product_ids":["BTC-USD"]}]}"#);
}

#[test]
fn test_subscribe_auth() {
    delay();
    let s = Subscribe {
        _type: SubscribeCmd::Subscribe,
        product_ids: vec!["BTC-USD".to_string()],
        channels: vec![
            Channel::Name(ChannelType::Heartbeat),
            Channel::WithProduct {
                name: ChannelType::Level2,
                product_ids: vec!["BTC-USD".to_string()],
            },
        ],
        auth: Some(Auth {
            signature: "111".to_string(),
            timestamp: "123".to_string(),
            passphrase: "333".to_string(),
            key: "000".to_string()
        })
    };

    let str = serde_json::to_string(&s).unwrap();
    assert_eq!(str,
               r#"{"type":"subscribe","product_ids":["BTC-USD"],"channels":["heartbeat",{"name":"level2","product_ids":["BTC-USD"]}],"signature":"111","key":"000","passphrase":"333","timestamp":"123"}"#);
}

#[test]
fn test_subscription() {
    delay();
    let stream = WSFeed::new(WS_SANDBOX_URL, &["BTC-USD"], &[ChannelType::Heartbeat]);
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
    delay();
    let found = Arc::new(AtomicBool::new(false));
    let found2 = found.clone();
    let stream = WSFeed::new(WS_SANDBOX_URL, &["BTC-USD"], &[ChannelType::Heartbeat]);
    let f = stream.take(3).for_each(move |msg| {
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
fn test_ticker() {
    delay();
    let found = Arc::new(AtomicBool::new(false));
    let found2 = found.clone();

    // hard to check in sandbox because low flow
    let stream = WSFeed::new(WS_URL, &["BTC-USD"], &[ChannelType::Ticker]);
    let f = stream.take(3).for_each(move |msg| {
        let str = format!("{:?}", msg);
        if str.contains("Ticker(Full { trade_id: ") {
            found2.swap(true, Ordering::Relaxed);
        }
        Ok(())
    });

    tokio::runtime::run(f.map_err(|e| println!("{:?}", e)));

    assert!(found.load(Ordering::Relaxed));
}

#[test]
fn test_level2() {
    delay();
    let found_snapshot = Arc::new(AtomicBool::new(false));
    let found_snapshot_2 = found_snapshot.clone();
    let found_l2update = Arc::new(AtomicBool::new(false));
    let found_l2update_2 = found_l2update.clone();

    // hard to check in sandbox because low flow
    let stream = WSFeed::new(WS_URL, &["BTC-USD"], &[ChannelType::Level2]);
    let f = stream.take(3).for_each(move |msg| {
        let str = format!("{:?}", msg);
        if str.starts_with("Level2(Snapshot { product_id: \"BTC-USD\", bids: [Level2SnapshotRecord")
            && !found_l2update_2.load(Ordering::Relaxed)
        {
            found_snapshot_2.swap(true, Ordering::Relaxed);
        } else if str
            .starts_with("Level2(L2update { product_id: \"BTC-USD\", changes: [Level2UpdateRecord")
        {
            found_l2update_2.swap(true, Ordering::Relaxed);
        }
        Ok(())
    });

    tokio::runtime::run(f.map_err(|e| println!("{:?}", e)));

    assert!(found_snapshot.load(Ordering::Relaxed));
    assert!(found_l2update.load(Ordering::Relaxed));
}

#[test]
fn test_match() {
    delay();
    let found_match = Arc::new(AtomicBool::new(false));
    let found_match_2 = found_match.clone();
    let found_full = Arc::new(AtomicBool::new(false));
    let found_full_2 = found_full.clone();

    // hard to check in sandbox because low flow
    let stream = WSFeed::new(WS_URL, &["BTC-USD"], &[ChannelType::Matches]);
    let f = stream.take(3).for_each(move |msg| {
        //            let str = format!("{:?}", msg);
        //            println!("{}", str);
        match msg {
            Message::Match(m) => {
                assert!(m.sequence > 0);
                found_match_2.swap(true, Ordering::Relaxed);
            }
            Message::Full(Full::Match(m)) => {
                assert!(m.trade_id > 0);
                found_full_2.swap(true, Ordering::Relaxed);
            }
            Message::Subscriptions { .. } => (),
            _ => assert!(false),
        };
        Ok(())
    });

    tokio::runtime::run(f.map_err(|e| println!("{:?}", e)));

    assert!(found_match.load(Ordering::Relaxed));
    assert!(found_full.load(Ordering::Relaxed));
}

#[test]
fn test_full() {
    delay();
    let found_received_limit = Arc::new(AtomicBool::new(false));
    let found_received_limit_2 = found_received_limit.clone();
    let _found_received_market = Arc::new(AtomicBool::new(false));
    let found_received_market_2 = found_received_limit.clone();
    let found_open = Arc::new(AtomicBool::new(false));
    let found_open_2 = found_open.clone();
    let found_done_limit = Arc::new(AtomicBool::new(false));
    let found_done_limit_2 = found_done_limit.clone();
    let found_done_market = Arc::new(AtomicBool::new(false));
    let found_done_market_2 = found_done_market.clone();
    let found_match = Arc::new(AtomicBool::new(false));
    let found_match_2 = found_match.clone();

    // hard to check in sandbox because low flow
    let stream = WSFeed::new(WS_URL, &["BTC-USD"], &[ChannelType::Full]);
    let f = stream.take(3000).for_each(move |msg| {
        let str = format!("{:?}", msg);
        if str.starts_with("Subscriptions { channels: [WithProduct { name: Full, product_ids") {
            ()
        } else if str.starts_with("Full(Match(Match { trade_id: ") {
            found_match_2.swap(true, Ordering::Relaxed);
        } else if str.starts_with("Full(Done(Limit { time: ") {
            found_done_limit_2.swap(true, Ordering::Relaxed);
        } else if str.starts_with("Full(Done(Market { time: ") {
            found_done_market_2.swap(true, Ordering::Relaxed);
        } else if str.starts_with("Full(Received(Limit") {
            found_received_limit_2.swap(true, Ordering::Relaxed);
        } else if str.starts_with("Full(Received(Market") {
            found_received_market_2.swap(true, Ordering::Relaxed);
        } else if str.starts_with("Full(Open(Open { time: ") {
            found_open_2.swap(true, Ordering::Relaxed);
        } else {
            println!("{}", str);
        }
        Ok(())
    });

    tokio::runtime::run(f.map_err(|e| println!("{:?}", e)));

    assert!(found_received_limit.load(Ordering::Relaxed));
    //    assert!(_found_received_market.load(Ordering::Relaxed));
    assert!(found_match.load(Ordering::Relaxed));
    assert!(found_done_limit.load(Ordering::Relaxed));
    assert!(found_done_market.load(Ordering::Relaxed));
    assert!(found_open.load(Ordering::Relaxed));
}


#[test]
fn test_user() {
    use coinbase_pro_rs::{Private, ASync, SANDBOX_URL, WSError};

    delay();


    let found_received = Arc::new(AtomicBool::new(false));
    let found_received_2 = found_received.clone();

    let stream = WSFeed::new_with_auth(WS_SANDBOX_URL, &["BTC-USD"], &[ChannelType::User],
        KEY, SECRET, PASSPHRASE);

    let f = stream.take(2).for_each(move |msg| {
        let str = format!("{:?}", msg);
        if str.contains("Subscriptions") {
            let client: Private<ASync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
            let res = client.buy_limit("BTC-USD", 0.001, 100.0, true, None)
                .and_then(|_| {
                    Ok(())
                })
                .map_err(|_| {
                    WSError::Read(tokio_tungstenite::tungstenite::Error::Utf8) // hm
                });
            futures::future::Either::A(res)
        } else {
            if str.contains("Full(Received(Limit")
                && str.contains("price: 100.0") {
                found_received_2.swap(true, Ordering::Relaxed);

            }
            futures::future::Either::B(futures::done(Ok(())))
        }
    });

    tokio::runtime::run(f.map_err(|e| println!("{:?}", e)));

    assert!(found_received.load(Ordering::Relaxed))
}


