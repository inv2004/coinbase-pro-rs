use coinbase_pro_rs::{structs::wsfeed::*, CBError, WSFeed, WS_SANDBOX_URL};
use future::ready;
use futures::{future, StreamExt};

#[tokio::main]
async fn main() {
    let stream = WSFeed::new(WS_SANDBOX_URL, &["BTC-USD"], &[ChannelType::Heartbeat]);

    stream
        .take(10)
        .for_each(|msg: Result<Message, CBError>| {
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
