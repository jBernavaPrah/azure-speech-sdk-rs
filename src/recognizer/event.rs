use serde_json::Value;
use uuid::Uuid;
use crate::connector::message::Message;
use std::fmt;


#[derive(Debug)]
/// Event for the speech recognition
pub enum Event<T> {
    /// Base event.
    Base(EventBase),
    /// Specific event.
    Specific(T),
}

#[derive(Debug)]
pub(crate) enum EventError {
    NoPath,
    Unprocessable,
    Skip,
}

pub(crate) trait FromMessage<T> {
    fn from_message(value: &Message) -> Result<Event<T>, EventError>;
}

#[derive(Debug)]
/// Base event for the speech recognition
pub enum EventBase {
    /// The hole conversation was cancelled.
    /// Stop and retry the conversation. If the error persists, open an issue in GitHub.
    Cancelled {
        /// The reason for the cancellation.
        reason: CancelledReason
    },
    SpeechStartDetected { offset: u32 },
    SpeechEndDetected { offset: u32 },
    SessionStarted { session_id: Uuid },
    SessionStopped { session_id: Uuid },
}

impl<T> FromMessage<T> for EventBase {
    fn from_message(message: &Message) -> Result<Event<T>, EventError> {
        if message.is_binary() || message.json().is_none() {
            return Err(EventError::Unprocessable);
        }

        let json_message = message.json().unwrap();

        match message.path().unwrap().as_str() {
            "speech.startdetected" => Ok(Event::Base(EventBase::SpeechStartDetected {
                offset: json_message.get("Offset").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
            })),
            "speech.enddetected" => Ok(Event::Base(EventBase::SpeechEndDetected {
                offset: json_message.get("Offset").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
            })),
            _ => Err(EventError::NoPath)
        }
    }
}

#[derive(Debug)]
/// Reason for the cancellation of the conversation.
/// Attention. Not all reasons are implemented yet.
pub enum CancelledReason {
    /// Indicates an authentication error.
    AuthenticationFailure,

    /// Indicates that one or more recognition parameters are invalid.
    BadRequestParameters,

    /// Indicates that the number of parallel requests exceeded the number of allowed
    /// concurrent transcriptions for the subscription.
    TooManyRequests,

    /// Indicates a connection error.
    ConnectionFailure,

    /// Indicates a time-out error when waiting for response from service.
    ServiceTimeout,

    /// Indicates that an error is returned by the service.
    ServiceError,

    /// Indicates an unexpected runtime error.
    RuntimeError,

    /// Indicates an quota overrun on existing key.
    Forbidden,
}


impl fmt::Display for CancelledReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CancelledReason::Forbidden => write!(f, "The recognizer is using a free subscription that ran out of quota."),
            CancelledReason::BadRequestParameters => write!(f, "Invalid parameter or unsupported audio format in the request."),
            CancelledReason::TooManyRequests => write!(f, "The number of parallel requests exceeded the number of allowed concurrent transcriptions."),
            _ => write!(f, "The speech service encountered an internal error and could not continue."),
        }
    }
}