use coinbase_pro_rs::{ASync, Public, SANDBOX_URL};
use tokio_compat_02::FutureExt;

#[tokio::main]
async fn main() {
    let client: Public<ASync> = Public::new_with_keep_alive(SANDBOX_URL, false);
    // if keep_alive is not disables - tokio::run will hold the connection without exiting test
    let time = client.get_time().await.unwrap();
    println!("Coinbase.time: {}", time.iso);
}
