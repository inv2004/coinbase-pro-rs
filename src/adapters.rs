extern crate tokio;

use hyper::rt::{Future, Stream};
use super::error::CBError;

pub trait Adapter<T>{
    type Result;
    fn process<F>(f: F) -> Self::Result
        where F: Future<Item = T, Error = CBError> + Send + 'static;
}

pub struct Sync;

impl<T> Adapter<T> for Sync {
    type Result = Result<T, CBError>;
    fn process<F>(f: F) -> Self::Result
        where F: Future<Item = T, Error = CBError> + Send + 'static
    {
        let mut rt = tokio::runtime::current_thread::Runtime::new().unwrap();
        rt.block_on(f)
    }
}

pub struct ASync;

impl<T> Adapter<T> for ASync {
    type Result = Box<Future<Item = T, Error = CBError> + Send>;
    fn process<F>(f: F) -> Self::Result
        where F: Future<Item = T, Error = CBError> + Send + 'static
    {
        Box::new(f)
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn test_sync() {
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
        rt.block_on(time);
    }
}

