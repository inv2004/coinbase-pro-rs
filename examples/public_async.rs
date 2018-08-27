extern crate tokio;
extern crate coinbase_pro_rs;

use hyper::rt::{Future, Stream};
use tokio::prelude::*;
use coinbase_pro_rs::{Public, ASync, SANDBOX_URL};

fn main() {
    let client: Public<ASync> = Public::new(SANDBOX_URL);
    let f = client.get_time()
        .map_err(|_| ())
        .and_then(|time| {
            println!("Coinbase.time: {}", time.iso);
            Ok(())
        });

    tokio::run(f);
}
