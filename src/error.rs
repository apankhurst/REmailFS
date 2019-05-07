use std::error::Error as StdError;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {

}
