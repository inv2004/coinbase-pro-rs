extern crate coinbase_pro_rs;
extern crate hyper;
extern crate tokio;

use coinbase_pro_rs::{ASync, Public, SANDBOX_URL};
use hyper::rt::Future;

fn main() {
    let client: Public<ASync> = Public::new_with_keep_alive(SANDBOX_URL, false);
    // if keep_alive is not disables - tokio::run will hold the connection without exiting test
    let f = client.get_time().map_err(|_| ()).and_then(|time| {
        println!("Coinbase.time: {}", time.iso);
        Ok(())
    });

    tokio::run(f);
}
