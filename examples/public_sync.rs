use coinbase_pro_rs::{Public, Sync, SANDBOX_URL};

fn main() {
    let client: Public<Sync> = Public::new(SANDBOX_URL);
    let time = client.get_time().unwrap();
    println!("Coinbase.time: {}", time.iso);
}
