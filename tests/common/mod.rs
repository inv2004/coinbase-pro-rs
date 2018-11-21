extern crate pretty_env_logger;
use std::{thread, time};

static DELAY_TIMEOUT: u64 = 200;

pub fn delay() {
    thread::sleep(time::Duration::from_millis(DELAY_TIMEOUT));
    pretty_env_logger::init();
}
