extern crate coinbase_pro_rs;

use coinbase_pro_rs::{Public, Sync};

fn main() {
    let client: Public<Sync> = Public::new();
    let time = client.get_time().unwrap();
    println!("Coinbase.time: {}", time.iso);
}