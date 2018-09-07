extern crate coinbase_pro_rs;
extern crate futures;
extern crate tokio;

use coinbase_pro_rs::structs::wsfeed::*;
use coinbase_pro_rs::{WSFeed, WS_SANDBOX_URL};
use futures::{Future, Stream};

fn main() {
    let stream = WSFeed::new(WS_SANDBOX_URL, &["BTC-USD"], &[ChannelType::Heartbeat]);

    let f = stream.take(10).for_each(|msg| {
        match msg {
            Message::Heartbeat {
                sequence,
                last_trade_id,
                time,
                ..
            } => println!("{}: seq:{} id{}", time, sequence, last_trade_id),
            Message::Error { message } => println!("Error: {}", message),
            Message::InternalError(_) => panic!("internal_error"),
            other => println!("{:?}", other),
        }
        Ok(())
    });

    tokio::run(f.map_err(|_| panic!("stream fail")));
}
