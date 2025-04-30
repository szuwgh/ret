use std::{io::Error as IOError, num::ParseIntError};
use thiserror::Error;
use url::ParseError;
pub type RetResult<T> = Result<T, RetError>;
use std::net::AddrParseError;
#[derive(Error, Debug)]
pub enum RetError {
    #[error("Unexpected: {0}")]
    Unexpected(String),
    #[error("Unexpected IO: {0}")]
    UnexpectIO(IOError),
    #[error("Parse url error: {0}")]
    UrlParseError(ParseError),
    #[error("Addr parse url error: {0}")]
    AddrParseError(String),
    #[error("Parse int error: {0}")]
    ParseIntError(ParseIntError),
}

impl From<ParseError> for RetError {
    fn from(e: ParseError) -> Self {
        RetError::UrlParseError(e)
    }
}

impl From<ParseIntError> for RetError {
    fn from(e: ParseIntError) -> Self {
        RetError::ParseIntError(e)
    }
}
