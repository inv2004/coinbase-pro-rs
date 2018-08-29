extern crate url;

pub const WS_SANDBOX_URL: &str = "wss://ws-feed-public.sandbox.pro.coinbase.com";

use futures::{Future, Stream, Sink};
use tokio_tungstenite::connect_async;
use serde_json;
use self::url::Url;

use error::WSError;
use structs::ws::*;
use super::tokio_tungstenite::tungstenite::Message;

struct WSFeed;

impl WSFeed {
    pub fn new(uri: &str, product_ids: &[&str], channels: &[ChannelType])
        -> impl Future<Item = (), Error = WSError>
    {
        let subscribe = Subscribe {
            _type: SubscribeCmd::Subscribe,
            product_ids: product_ids.into_iter().map(|x| x.to_string()).collect(),
            channels: channels.to_vec().into_iter().map(|x| Channel::Name(x)).collect::<Vec<_>>()
        };

        Self::new_with_sub(uri, subscribe)
    }

    pub fn new_with_sub(uri: &str, subsribe: Subscribe) -> impl Future<Item = (), Error = WSError> {
        let url = Url::parse(uri).unwrap();

        connect_async(url)
            .map_err(WSError::Connect)
            .and_then(move |(ws_stream, _)| {
                println!("WebSocket handshake has been successfully completed");
                let (sink, stream) = ws_stream.split();

                let subsribe = serde_json::to_string(&subsribe).unwrap();

                sink.send(Message::Text(subsribe))
                    .map_err(WSError::Send)
                    .and_then(|_| {
                        stream
                            .map_err(WSError::Read)
                            .for_each(move |msg| {
                                println!("{:?}", msg);
                                Ok(())
                            })
                    })
            })
    }
}

#[cfg(test)]
mod tests {
    extern crate tokio;

    use super::*;

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
    fn test_ws() {
        let f = WSFeed::new(WS_SANDBOX_URL,
            &["BTC-USD"], &[ChannelType::Heartbeat]);
        tokio::runtime::run(f.map_err(|_| ()));
    }
}

