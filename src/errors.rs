//! # Errors

use tokio::task::JoinError;

use tokio_tungstenite::tungstenite::{Error as TError};
use crate::connector::message;

#[derive(Debug)]
/// Error struct, used to represent errors in the library.
pub struct Error {
    /// Error message
    pub message: String,
    /// Optionally the Error code
    pub code: Option<String>,
}

impl Error {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Error {
            message: message.into(),
            code: None,
        }
    }
    #[allow(dead_code)]
    pub(crate) fn new_with_code(message: impl Into<String>, code: impl Into<String>) -> Self {
        Error {
            message: message.into(),
            code: Some(code.into()),
        }
    }
}


impl From<JoinError> for Error {
    fn from(error: JoinError) -> Self {
        Error::new(format!("{}", error)) 
    }
}

impl From<tokio::sync::mpsc::error::SendError<message::Message>> for Error {
    fn from(error: tokio::sync::mpsc::error::SendError<message::Message>) -> Self {
        Error::new(format!("{}", error))
    }

}

impl From<TError> for Error {
    fn from(error: TError) -> Self {
        Error::new(format!("{}", error))
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(error: serde_json::error::Error) -> Error {
        Error::new(format!("{}", error))
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::new(format!("{}", error))
    }
}

