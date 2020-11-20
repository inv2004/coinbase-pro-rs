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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        structs::reqs::{self, OrderTimeInForce, OrderTimeInForceCancelAfter},
        utils::delay,
        Public, SANDBOX_URL,
    };
    use futures::{future, TryFutureExt};

    #[test]
    fn test_sync() {
        delay();
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
        delay();
        let client: Public<ASync> = Public::new(SANDBOX_URL);
        let time = client.get_time().and_then(|time| {
            let time_str = format!("{:?}", time);
            assert!(time_str.starts_with("Time {"));
            assert!(time_str.contains("iso:"));
            assert!(time_str.contains("epoch:"));
            assert!(time_str.ends_with("}"));
            future::ready(Ok(()))
        });
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        rt.block_on(time).ok();
    }
}
