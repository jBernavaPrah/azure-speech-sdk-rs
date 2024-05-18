use std::result;
use tokio::task::JoinError;

use tokio_tungstenite::tungstenite::{Error as TError};
use crate::connector::message;


pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub code: Option<String>,
}

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Error {
            message: message.into(),
            code: None,
        }
    }
    pub fn new_with_code(message: impl Into<String>, code: impl Into<String>) -> Self {
        Error {
            message: message.into(),
            code: Some(code.into()),
        }
    }
}


impl From<JoinError> for Error {
    fn from(error: JoinError) -> Self {
        Error {
            message: format!("{}", error),
            code: None,
        }
    }
}

impl From<tokio::sync::mpsc::error::SendError<message::Message>> for Error {
    fn from(error: tokio::sync::mpsc::error::SendError<message::Message>) -> Self {
        Error {
            message: format!("{}", error),
            code: None,
        }
    }

}

impl From<TError> for Error {
    fn from(error: TError) -> Self {
        Error {
            message: format!("{}", error),
            code: None,
        }
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(error: serde_json::error::Error) -> Error {
        Error {
            message: format!("{}", error),
            code: None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error {
            message: format!("{}", error),
            code: None,
        }
    }
}

