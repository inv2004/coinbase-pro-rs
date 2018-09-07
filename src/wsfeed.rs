//! Contains structure which provides futures::Stream to websocket-feed of Coinbase api

extern crate url;

use self::url::Url;
use futures::{Future, Sink, Stream};
use serde_json;
use tokio_tungstenite::connect_async;

use super::tokio_tungstenite::tungstenite::Message as TMessage;
use error::WSError;
use structs::wsfeed::*;

pub struct WSFeed;

fn convert_msg(msg: TMessage) -> Message {
    match msg {
        TMessage::Text(str) => serde_json::from_str(&str).unwrap_or_else(|e| {
            Message::InternalError(WSError::Serde {
                error: e,
                data: str,
            })
        }),
        _ => unreachable!(), // filtered in stream
    }
}

impl WSFeed {
    // Constructor for simple subcription with product_ids and channels
    pub fn new(
        uri: &str,
        product_ids: &[&str],
        channels: &[ChannelType],
    ) -> impl Stream<Item = Message, Error = WSError> {
        let subscribe = Subscribe {
            _type: SubscribeCmd::Subscribe,
            product_ids: product_ids.into_iter().map(|x| x.to_string()).collect(),
            channels: channels
                .to_vec()
                .into_iter()
                .map(|x| Channel::Name(x))
                .collect::<Vec<_>>(),
        };

        Self::new_with_sub(uri, subscribe)
    }

    // Constructor for extended subcription via Subscribe structure
    pub fn new_with_sub(
        uri: &str,
        subsribe: Subscribe,
    ) -> impl Stream<Item = Message, Error = WSError> {
        let url = Url::parse(uri).unwrap();

        connect_async(url)
            .map_err(WSError::Connect)
            .and_then(move |(ws_stream, _)| {
                debug!("WebSocket handshake has been successfully completed");
                let (sink, stream) = ws_stream.split();

                let subsribe = serde_json::to_string(&subsribe).unwrap();

                sink.send(TMessage::Text(subsribe))
                    .map_err(WSError::Send)
                    .and_then(|_| {
                        debug!("subsription sent");
                        let stream = stream
                            .filter(|msg| msg.is_text())
                            .map_err(WSError::Read)
                            .map(convert_msg);
                        Ok(stream)
                    })
            }).flatten_stream()
    }
}
