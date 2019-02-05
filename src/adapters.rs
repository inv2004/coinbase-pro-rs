extern crate tokio;

use super::error::CBError;
use hyper::rt::Future;
use tokio::runtime::current_thread::Runtime;
use std::cell::RefCell;

pub trait Adapter<T> {
    type Result;
    fn process<F>(f: F) -> Self::Result
    where
        F: Future<Item = T, Error = CBError> + Send + 'static;
}

pub struct Sync;

thread_local! {
    static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime::new().unwrap());
}

impl<T> Adapter<T> for Sync {
    type Result = Result<T, CBError>;
    fn process<F>(f: F) -> Self::Result
    where
        F: Future<Item = T, Error = CBError> + Send + 'static,
    {
        RUNTIME.with(|rt| {
            rt.borrow_mut().block_on(f)
        })

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

