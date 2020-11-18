extern crate coinbase_pro_rs;
extern crate futures;
extern crate tokio;

use coinbase_pro_rs::structs::wsfeed::*;
use coinbase_pro_rs::{WSError, WSFeed, WS_SANDBOX_URL};
use future::ready;
use futures::{future, Stream, StreamExt, TryStreamExt};
use std::future::Future;

#[tokio::main]
async fn main() {
    let stream = WSFeed::new(WS_SANDBOX_URL, &["BTC-USD"], &[ChannelType::Heartbeat]);

    let stream = stream.take(10);
    stream
        .for_each(|msg: Result<Message, WSError>| {
            match msg.unwrap() {
                Message::Heartbeat {
                    sequence,
                    last_trade_id,
                    time,
                    ..
                } => println!("{}: seq:{} id{}", time, sequence, last_trade_id),
                Message::Error { message } => println!("Error: {}", message),
                Message::InternalError(_) => panic!("internal_error"),
                other => println!("{:?}", other),
            };
            ready(())
        })
        .await;

    // f.map_err(|_| panic!("stream fail"));
}
