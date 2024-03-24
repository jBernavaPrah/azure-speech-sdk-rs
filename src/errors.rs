use std::result;
use tokio::task::JoinError;

use tokio_tungstenite::tungstenite::Error as TError;

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub code: Option<String>,
}

impl Error {
    pub fn new(message: String) -> Self {
        Error {
            message,
            code: None,
        }
    }
    pub fn new_with_code(message: String, code: String) -> Self {
        Error {
            message,
            code: Some(code),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

impl From<JoinError> for Error {
    fn from(error: JoinError) -> Self {
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