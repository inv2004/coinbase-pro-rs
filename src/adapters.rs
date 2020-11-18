extern crate tokio;

use super::error::CBError;
use std::future::Future;

use std::cell::RefCell;
use std::fmt::Debug;
use std::{io, pin::Pin};
use tokio::runtime::Runtime;
use tokio_compat_02::FutureExt;

pub trait Adapter<T> {
    type Result: Sized;
    fn process<F>(&self, f: F) -> Self::Result
    where
        F: Future<Output = Result<T, CBError>> + 'static;
}

pub trait AdapterNew: Sized {
    type Error: Debug;
    fn new() -> Result<Self, Self::Error>;
}

pub struct Sync(RefCell<Runtime>);

impl AdapterNew for Sync {
    type Error = io::Error;
    fn new() -> Result<Self, Self::Error> {
        Ok(Sync(RefCell::new(Runtime::new()?)))
    }
}

impl<T> Adapter<T> for Sync
where
    T: Send + 'static,
{
    type Result = Result<T, CBError>;
    fn process<F>(&self, f: F) -> Self::Result
    where
        F: Future<Output = Result<T, CBError>> + 'static,
    {
        self.0.borrow_mut().block_on(f.compat())
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
    type Result = Pin<Box<dyn Future<Output = Result<T, CBError>>>>;

    fn process<F>(&self, f: F) -> Self::Result
    where
        F: Future<Output = Result<T, CBError>> + 'static,
    {
        Box::pin(f.compat())
    }
}
