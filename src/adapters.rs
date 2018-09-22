extern crate tokio;

use super::error::CBError;
use hyper::rt::Future;

pub trait Adapter<T> {
    type Result;
    fn process<F>(f: F) -> Self::Result
    where
        F: Future<Item = T, Error = CBError> + Send + 'static;
}

pub struct Sync;

impl<T> Adapter<T> for Sync {
    type Result = Result<T, CBError>;
    fn process<F>(f: F) -> Self::Result
    where
        F: Future<Item = T, Error = CBError> + Send + 'static,
    {
        let mut rt = tokio::runtime::current_thread::Runtime::new().unwrap();
        rt.block_on(f)
    }
}

pub struct ASync;

impl<T> Adapter<T> for ASync {
    type Result = Box<Future<Item = T, Error = CBError> + Send>;
    fn process<F>(f: F) -> Self::Result
    where
        F: Future<Item = T, Error = CBError> + Send + 'static,
    {
        Box::new(f)
    }
}

