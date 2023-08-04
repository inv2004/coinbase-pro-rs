//! Contains structure which provides futures::Stream to websocket-feed of Coinbase api

use async_trait::async_trait;
use futures::{future, Sink, Stream};
use futures_util::{sink::SinkExt, stream::TryStreamExt};
use hyper::Method;
use serde_json;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_tungstenite::{connect_async, tungstenite::Message as TMessage};
use url::Url;

use crate::{private::Private, structs::wsfeed::*, ASync, CBError, WSError};

pub struct WSFeed;

fn convert_msg(msg: TMessage) -> Message {
    match msg {
        TMessage::Text(str) => serde_json::from_str(&str).unwrap_or_else(|e| {
            Message::InternalError(CBError::Serde {
                error: e,
                data: str,
            })
        }),
        _ => unreachable!(), // filtered in stream
    }
}

impl WSFeed {
    // Constructor for simple subcription with product_ids and channels
    pub async fn connect(
        uri: &str,
        product_ids: &[&str],
        channels: &[ChannelType],
    ) -> Result<impl CBStream + CBSink, CBError> {
        let subscribe = Subscribe {
            _type: SubscribeCmd::Subscribe,
            product_ids: product_ids.into_iter().map(|x| x.to_string()).collect(),
            channels: channels
                .to_vec()
                .into_iter()
                .map(|x| Channel::Name(x))
                .collect::<Vec<_>>(),
            auth: None,
        };

        Self::connect_with_sub(uri, subscribe).await
    }

    // Constructor for extended subcription via Subscribe structure
    pub async fn connect_with_sub(
        uri: &str,
        subscribe: Subscribe,
    ) -> Result<impl CBStream + CBSink, CBError> {
        let url = Url::parse(uri).unwrap();

        let stream = connect_async(url)
            .await
            .map_err(|e| CBError::Websocket(WSError::Connect(e)))?
            .0;
        log::debug!("WebSocket handshake has been successfully completed");

        let mut stream = stream
            .try_filter(|msg| future::ready(msg.is_text()))
            .map_ok(convert_msg)
            .sink_map_err(|e| CBError::Websocket(WSError::Send(e)))
            .map_err(|e| CBError::Websocket(WSError::Read(e)));

        let subscribe = serde_json::to_string(&subscribe).unwrap();

        stream.send(TMessage::Text(subscribe)).await?;
        log::debug!("subsription sent");

        Ok(stream)
    }

    // Constructor for simple subcription with product_ids and channels with auth
    pub async fn connect_with_auth(
        uri: &str,
        product_ids: &[&str],
        channels: &[ChannelType],
        key: &str,
        secret: &str,
        passphrase: &str,
    ) -> Result<impl CBStream + CBSink, CBError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("leap-second")
            .as_secs();

        let signature =
            Private::<ASync>::sign(secret, timestamp, Method::GET, "/users/self/verify", "");

        let auth = Auth {
            signature,
            key: key.to_string(),
            passphrase: passphrase.to_string(),
            timestamp: timestamp.to_string(),
        };

        let subscribe = Subscribe {
            _type: SubscribeCmd::Subscribe,
            product_ids: product_ids.into_iter().map(|x| x.to_string()).collect(),
            channels: channels
                .to_vec()
                .into_iter()
                .map(|x| Channel::Name(x))
                .collect::<Vec<_>>(),
            auth: Some(auth),
        };

        Self::connect_with_sub(uri, subscribe).await
    }
}

impl<T> CBSink for T where T: Sink<TMessage, Error = CBError> + Unpin + Send {}

#[async_trait]
pub trait CBSink: Sink<TMessage, Error = CBError> + Unpin + Send {
    async fn subscribe(
        &mut self,
        product_ids: &[&str],
        channels: &[ChannelType],
        auth: Option<Auth>,
    ) -> Result<(), CBError> {
        let subscribe = Subscribe {
            _type: SubscribeCmd::Subscribe,
            product_ids: product_ids.into_iter().map(|x| x.to_string()).collect(),
            channels: channels
                .to_vec()
                .into_iter()
                .map(|x| Channel::Name(x))
                .collect::<Vec<_>>(),
            auth,
        };
        let subscribe = serde_json::to_string(&subscribe).unwrap();
        self.send(TMessage::Text(subscribe)).await
    }
}

impl<T> CBStream for T where T: Stream<Item = Result<Message, CBError>> + Unpin + Send {}

#[async_trait]
pub trait CBStream: Stream<Item = Result<Message, CBError>> + Unpin + Send {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{utils::delay, WSFeed, WS_SANDBOX_URL, WS_URL};
    use futures::{
        future,
        stream::{StreamExt, TryStreamExt},
    };
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    static KEY: &str = "9eaa4603717ffdc322771a933ae12501";
    static SECRET: &str =
        "RrLem7Ihmnn57ryW4Cc3Rp31h+Bm2DEPmzNbRiPrQQRE1yH6WNybmhK8xSqHjUNaR/V8huS+JMhBlr8PKt2GhQ==";
    static PASSPHRASE: &str = "sandbox";

    #[test]
    fn test_subscribe() {
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
            auth: None,
        };

        let str = serde_json::to_string(&s).unwrap();
        assert_eq!(
            str,
            r#"{"type":"subscribe","product_ids":["BTC-USD"],"channels":["heartbeat",{"name":"level2","product_ids":["BTC-USD"]}]}"#
        );
    }

    #[test]
    fn test_status() {
        let s = Subscribe {
            _type: SubscribeCmd::Subscribe,
            product_ids: vec!["BTC-USD".to_string()],
            channels: vec![Channel::Name(ChannelType::Status)],
            auth: None,
        };

        let str = serde_json::to_string(&s).unwrap();
        assert_eq!(
            str,
            r#"{"type":"subscribe","product_ids":["BTC-USD"],"channels":["status"]}"#
        );
    }
    #[tokio::test]
    #[serial]
    async fn test_status_stream() {
        delay();
        let found = Arc::new(AtomicBool::new(false));
        let found2 = found.clone();
        let stream = WSFeed::connect(WS_SANDBOX_URL, &["BTC-USD"], &[ChannelType::Status])
            .await
            .unwrap();
        stream
            .take(2)
            .try_for_each(|msg| {
                println!("{:?}", msg);
                let str = format!("{:?}", msg);
                if str.contains("Status { products:") {
                    found2.swap(true, Ordering::Relaxed);
                }
                future::ready(Ok(()))
            })
            .await
            .map_err(|e| println!("{:?}", e))
            .unwrap();

        assert!(found.load(Ordering::Relaxed));
    }

    #[test]
    fn test_subscribe_auth() {
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
                key: "000".to_string(),
            }),
        };

        let str = serde_json::to_string(&s).unwrap();
        assert_eq!(
            str,
            r#"{"type":"subscribe","product_ids":["BTC-USD"],"channels":["heartbeat",{"name":"level2","product_ids":["BTC-USD"]}],"signature":"111","key":"000","passphrase":"333","timestamp":"123"}"#
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_subscription() {
        delay();
        let stream = WSFeed::connect(WS_SANDBOX_URL, &["BTC-USD"], &[ChannelType::Heartbeat])
            .await
            .unwrap();
        stream
            .take(1)
            .try_for_each(|msg| {
                assert_eq!(
                    &msg,
                    &Message::Subscriptions {
                        channels: vec![Channel::WithProduct {
                            name: ChannelType::Heartbeat,
                            product_ids: vec!["BTC-USD".to_string()]
                        }]
                    }
                );
                future::ready(Ok(()))
            })
            .await
            .map_err(|e| println!("{:?}", e))
            .unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_heartbeat() {
        delay();
        let found = Arc::new(AtomicBool::new(false));
        let found2 = found.clone();
        let stream = WSFeed::connect(WS_SANDBOX_URL, &["BTC-USD"], &[ChannelType::Heartbeat])
            .await
            .unwrap();
        stream
            .take(3)
            .try_for_each(move |msg| {
                let str = format!("{:?}", msg);
                if str.starts_with("Heartbeat { sequence: ") {
                    found2.swap(true, Ordering::Relaxed);
                }
                future::ready(Ok(()))
            })
            .await
            .map_err(|e| println!("{:?}", e))
            .unwrap();

        assert!(found.load(Ordering::Relaxed));
    }

    #[tokio::test]
    #[serial]
    async fn test_ticker() {
        delay();
        let found = Arc::new(AtomicBool::new(false));
        let found2 = found.clone();

        // hard to check in sandbox because low flow
        let stream = WSFeed::connect(WS_URL, &["BTC-USD"], &[ChannelType::Ticker])
            .await
            .unwrap();
        stream
            .take(3)
            .try_for_each(move |msg| {
                let str = format!("{:?}", msg);
                if str.contains("Ticker(Full { trade_id: ") {
                    found2.swap(true, Ordering::Relaxed);
                }
                future::ready(Ok(()))
            })
            .await
            .unwrap();

        assert!(found.load(Ordering::Relaxed));
    }

    #[tokio::test]
    #[serial]
    async fn test_ticker_batch() {
        delay();
        let found = Arc::new(AtomicBool::new(false));
        let found2 = found.clone();

        // hard to check in sandbox because low flow
        let stream = WSFeed::connect(WS_URL, &["BTC-USD"], &[ChannelType::TickerBatch])
            .await
            .unwrap();
        stream
            .take(3)
            .try_for_each(move |msg| {
                let str = format!("{:?}", msg);
                if str.contains("Ticker(Full { trade_id: ") {
                    found2.swap(true, Ordering::Relaxed);
                }
                future::ready(Ok(()))
            })
            .await
            .unwrap();

        assert!(found.load(Ordering::Relaxed));
    }

    #[tokio::test]
    #[serial]
    async fn test_level2() {
        delay();
        let found_snapshot = Arc::new(AtomicBool::new(false));
        let found_snapshot_2 = found_snapshot.clone();
        let found_l2update = Arc::new(AtomicBool::new(false));
        let found_l2update_2 = found_l2update.clone();

        // hard to check in sandbox because low flow
        let stream = WSFeed::connect(WS_URL, &["BTC-USD"], &[ChannelType::Level2])
            .await
            .unwrap();
        stream
            .take(3)
            .try_for_each(move |msg| {
                let str = format!("{:?}", msg);
                if str.starts_with(
                    "Level2(Snapshot { product_id: \"BTC-USD\", bids: [Level2SnapshotRecord",
                ) && !found_l2update_2.load(Ordering::Relaxed)
                {
                    found_snapshot_2.swap(true, Ordering::Relaxed);
                } else if str.starts_with(
                    "Level2(L2update { product_id: \"BTC-USD\", changes: [Level2UpdateRecord",
                ) {
                    found_l2update_2.swap(true, Ordering::Relaxed);
                }
                future::ready(Ok(()))
            })
            .await
            .map_err(|e| println!("{:?}", e))
            .unwrap();

        assert!(found_snapshot.load(Ordering::Relaxed));
        assert!(found_l2update.load(Ordering::Relaxed));
    }

    #[tokio::test]
    #[serial]
    async fn test_match() {
        delay();
        let found_match = Arc::new(AtomicBool::new(false));
        let found_match_2 = found_match.clone();
        let found_full = Arc::new(AtomicBool::new(false));
        let found_full_2 = found_full.clone();

        // hard to check in sandbox because low flow
        let stream = WSFeed::connect(WS_URL, &["BTC-USD"], &[ChannelType::Matches])
            .await
            .unwrap();
        let f = stream.take(3).try_for_each(move |msg| {
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
            future::ready(Ok(()))
        });

        f.await.map_err(|e| println!("{:?}", e)).unwrap();

        assert!(found_match.load(Ordering::Relaxed));
        assert!(found_full.load(Ordering::Relaxed));
    }

    #[tokio::test]
    #[ignore]
    #[serial]
    async fn test_full() {
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
        let stream = WSFeed::connect(WS_URL, &["BTC-USD"], &[ChannelType::Full])
            .await
            .unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            stream
                .take(10000)
                .try_for_each(move |msg| {
                    let str = format!("{:?}", msg);
                    if str.starts_with(
                        "Subscriptions { channels: [WithProduct { name: Full, product_ids",
                    ) {
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
                    future::ready(Ok(()))
                })
                .await
                .map_err(|e| println!("{:?}", e))
        })
        .unwrap();

        assert!(found_received_limit.load(Ordering::Relaxed));
        // assert!(_found_received_market.load(Ordering::Relaxed));
        assert!(found_match.load(Ordering::Relaxed));
        assert!(found_done_limit.load(Ordering::Relaxed));
        assert!(found_done_market.load(Ordering::Relaxed));
        assert!(found_open.load(Ordering::Relaxed));
    }

    #[tokio::test]
    #[serial]
    async fn test_user() {
        use crate::{ASync, Private, WSError, SANDBOX_URL};

        delay();

        let found_received = Arc::new(AtomicBool::new(false));
        let found_received_2 = found_received.clone();

        let stream = WSFeed::connect_with_auth(
            WS_SANDBOX_URL,
            &["BTC-USD"],
            &[ChannelType::User],
            KEY,
            SECRET,
            PASSPHRASE,
        )
        .await
        .unwrap();

        stream
            .take(2)
            .try_for_each(move |msg| {
                let found_received_2 = found_received_2.clone();
                async move {
                    match &msg {
                        Message::Subscriptions { .. } => {
                            let client: Private<ASync> =
                                Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
                            let res: Result<(), CBError> = client
                                .buy_limit("BTC-USD", 1_f64, 100.0_f64, true)
                                .await
                                .and_then(|_| Ok(()))
                                .map_err(|_| {
                                    CBError::Websocket(WSError::Read(
                                        tokio_tungstenite::tungstenite::Error::Utf8,
                                    ))
                                    // hm
                                });
                            res
                        }
                        Message::Full(Full::Received(Received::Limit { price, .. })) => {
                            if (price - 100.0).abs() < 0.00001 {
                                found_received_2.swap(true, Ordering::Relaxed);
                            }
                            Ok(())
                        }
                        _ => {
                            assert!(false);
                            Ok(())
                        }
                    }
                }
            })
            .await
            .unwrap();

        assert!(found_received.load(Ordering::Relaxed))
    }

    #[tokio::test]
    #[serial]
    async fn test_dynamic_subscription() {
        delay();

        let stream = WSFeed::connect(WS_URL, &[], &[]).await.unwrap();
        let (tx, mut rx) = stream.split();

        let pids = vec!["BTC-USD", "ETH-USD"];

        async fn subscribe_with_delay(mut tx: impl CBSink, pids: Vec<&str>) {
            for pid in pids {
                tx.subscribe(&[pid], &[ChannelType::Full], None)
                    .await
                    .unwrap();
                println!("{:?} subscription request sent.", pid);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }

        tokio::spawn(subscribe_with_delay(tx, pids.clone()));

        // current connect function emits empty subscription message with empty product_ids & channels.
        rx.next().await;

        let mut cnt = 0;
        while let Some(msg) = rx.next().await {
            let msg = msg.unwrap();
            if let Message::Subscriptions { ref channels } = msg {
                cnt += 1;
                assert_eq!(
                    channels,
                    &vec![Channel::WithProduct {
                        name: ChannelType::Full,
                        product_ids: pids[..cnt]
                            .iter()
                            .map(|pid| pid.to_string())
                            .collect::<Vec<_>>()
                    }]
                );

                if cnt == pids.len() {
                    break;
                }
            }
            println!("{:?}", msg);
        }
    }
}
