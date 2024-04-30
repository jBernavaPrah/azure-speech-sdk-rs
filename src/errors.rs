use std::result;
use cpal::{DefaultStreamConfigError, DeviceNameError, DevicesError};
use flume::SendError;
use tokio::task::JoinError;

use tokio_tungstenite::tungstenite::Error as TError;


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

impl From<DevicesError> for Error {
    fn from(error: DevicesError) -> Error {
        Error {
            message: format!("{}", error),
            code: None,
        }
    }
}

impl From<DeviceNameError> for Error {
    fn from(error: DeviceNameError) -> Error {
        Error {
            message: format!("{}", error),
            code: None,
        }
    }
}

impl From<DefaultStreamConfigError> for Error {
    fn from(error: DefaultStreamConfigError) -> Error {
        Error {
            message: format!("{}", error),
            code: None,
        }
    }
}

impl<T> From<SendError<T>> for Error {
    fn from(error: SendError<T>) -> Error {
        Error {
            message: format!("{}", error),
            code: None,
        }
    }
}