use std::fmt::Debug;
use std::result;
use std::sync::{PoisonError};
use async_channel::SendError;
use serde::Deserialize;

/// Result type for the library.
pub type Result<T> = result::Result<T, Error>;


#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
/// Error struct, used to represent errors in the library.
pub enum Error {
    IOError(String),
    InvalidResponse(String),
    ParseError(String),
    InternalError(String),
    RuntimeError(String),
    ServerDisconnect(String),
    Forbidden,
    TooManyRequests,
    BadRequest,
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Error::ParseError(e.to_string())
    }
}

impl<T: Debug> From<PoisonError<T>> for Error {
    fn from(e: PoisonError<T>) -> Self {
        Error::InternalError(e.to_string())
    }
}

impl From<serde_json::Error> for Error{
    fn from(e: serde_json::Error) -> Self {
        Error::ParseError(e.to_string())
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::InternalError(s.to_string())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::InternalError(s)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IOError(e.to_string())
    }
}

impl<T: Debug> From<SendError<T>> for Error {
    fn from(e: SendError<T>) -> Self {
        Error::InternalError(e.to_string())
    }
}