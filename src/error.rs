use imap;
use std::result;

pub type Result<T> = result::Result<T, Error>;

pub enum Error {
    IMAPError(imap::error::Error),
}

impl From<imap::error::Error> for Error {
    fn from(error: imap::error::Error) -> Self {
        Error::IMAPError(error)
    }
}
