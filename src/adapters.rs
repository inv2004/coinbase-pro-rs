extern crate tokio;

use super::error::CBError;
use hyper::rt::Future;

use std::cell::RefCell;
use tokio::runtime::current_thread::Runtime;
use std::io;
use std::fmt::Debug;

pub trait Adapter<T> {
    type Result;
    fn process<F>(&self, f: F) -> Self::Result
    where
        F: Future<Item = T, Error = CBError> + Send + 'static;
}

pub trait AdapterNew: Sized {
    type Error: Debug;
    fn new() -> Result<Self, Self::Error>;
}

pub struct Sync(RefCell<Runtime>);

impl AdapterNew for Sync {
    type Error = io::Error;
    fn new() -> Result<Self, Self::Error> {
        Ok(Sync(RefCell::new(
            Runtime::new()?
        )))
    }
}

impl<T> Adapter<T> for Sync {
    type Result = Result<T, CBError>;
    fn process<F>(&self, f: F) -> Self::Result
    where
        F: Future<Item = T, Error = CBError> + Send + 'static,
    {
        self.0.borrow_mut().block_on(f)
    }
}

pub struct ASync;

impl AdapterNew for ASync {
    type Error = ();
    fn new() -> Result<Self, Self::Error> {
        Ok(ASync)
    }
}

impl<T> Adapter<T> for ASync {
    type Result = Box<Future<Item = T, Error = CBError> + Send>;
    fn process<F>(&self, f: F) -> Self::Result
    where
        F: Future<Item = T, Error = CBError> + Send + 'static,
    {
        Box::new(f)
    }
}

