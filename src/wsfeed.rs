extern crate url;

pub const WS_SANDBOX_URL: &str = "wss://ws-feed-public.sandbox.pro.coinbase.com";

use futures::{Future, Stream};
use error::WSError;

use super::structs::ws;
use super::tokio_tungstenite::connect_async;
//use super::tokio_tungstenite::tungstenite::Message;
use self::url::Url;

struct WSFeed;

impl WSFeed {
    pub fn new(uri: &str) -> impl Future<Item = (), Error = WSError> {
        let url = Url::parse(uri).unwrap();

        connect_async(url)
            .map_err(WSError::Connect)
            .and_then(move |(ws_stream, _)| {
                println!("WebSocket handshake has been successfully completed");
                let (sink, stream) = ws_stream.split();
                Ok(())
            })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}



