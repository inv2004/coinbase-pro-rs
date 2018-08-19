use error::Result;
use structs::*;

pub trait Public {
    fn get_time(&self) -> Result<Time>;
    fn get_currencies(&self) -> Result<Vec<Currency>>;
}
