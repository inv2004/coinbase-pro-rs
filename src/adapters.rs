extern crate tokio;

use hyper::rt::{Future, Stream};
use super::error::CBError;

pub trait Adapter<T> {
    type Result;
    fn process(p: T) -> Self::Result;
}

pub struct Sync;

impl<T> Adapter<T> for Sync where T: Future {
    type Result = Result<T, CBError>;
    fn process(p: T) -> Self::Result
    {
        let mut rt = tokio::runtime::current_thread::Runtime::new().unwrap();
        rt.block_on(p)
    }
}

