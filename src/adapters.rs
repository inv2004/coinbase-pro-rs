use super::error::CBError;
use std::future::Future;

use std::cell::RefCell;
use std::fmt::Debug;
use std::{io, pin::Pin};
use tokio::runtime::Runtime;

pub trait Adapter<T> {
    type Result: Sized;
    fn process<F>(&self, f: F) -> Self::Result
    where
        F: Future<Output = Result<T, CBError>> + Send + 'static;
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
        F: Future<Output = Result<T, CBError>> + Send + 'static,
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
    type Result = Pin<Box<dyn Future<Output = Result<T, CBError>> + Send>>;

    fn process<F>(&self, f: F) -> Self::Result
    where
        F: Future<Output = Result<T, CBError>> + Send + 'static,
    {
        Box::pin(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{utils::delay, Public, SANDBOX_URL};
    use futures::{future, TryFutureExt};

    #[test]
    #[serial]
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
    #[serial]
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
        let rt = Runtime::new().unwrap();
        rt.block_on(time).ok();
    }
}
