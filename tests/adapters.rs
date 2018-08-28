extern crate tokio;
extern crate coinbase_pro_rs;

use tokio::prelude::Future;
use std::{thread, time};
use coinbase_pro_rs::*;

static DELAY_TIMEOUT: u64 = 200;

fn delay_ms(ms: u64) {
    thread::sleep(time::Duration::from_millis(ms));
}

#[test]
fn test_sync() {
    delay_ms(DELAY_TIMEOUT);
    let client: Public<Sync> = Public::new(SANDBOX_URL);
    let time = client.get_time().unwrap();
    let time_str = format!("{:?}", time);
    assert!(time_str.starts_with("Time {"));
    assert!(time_str.contains("iso:"));
    assert!(time_str.contains("epoch:"));
    assert!(time_str.ends_with("}"));
}

#[test]
fn test_async() {
    delay_ms(DELAY_TIMEOUT);
    let client: Public<ASync> = Public::new(SANDBOX_URL);
    let time = client.get_time()
        .and_then(|time| {
            let time_str = format!("{:?}", time);
            assert!(time_str.starts_with("Time {"));
            assert!(time_str.contains("iso:"));
            assert!(time_str.contains("epoch:"));
            assert!(time_str.ends_with("}"));
            Ok(())
        });
    let mut rt = tokio::runtime::current_thread::Runtime::new().unwrap();
    rt.block_on(time).ok();
}

