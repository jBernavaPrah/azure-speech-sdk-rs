use uuid::Uuid;
use std::fmt;
use crate::message::Message;

#[derive(Debug, Clone)]
/// Event for the speech recognition
pub enum Event<T> {
    /// Base event.
    Base(EventBase),
    /// Specific event.
    Specific(T),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Error {
    NoPath,
    Unprocessable,
    Skip,
    Error(CancelledReason),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NoPath => write!(f, "No path found in the message."),
            Error::Unprocessable => write!(f, "The message is unprocessable."),
            Error::Skip => write!(f, "The message should be skipped."),
            Error::Error(reason) => write!(f, "An error occurred: {}", reason),
        }
    }
}


#[derive(Debug, Clone)]
/// Base event for the speech recognition
pub enum EventBase {
    /// The hole conversation was cancelled.
    /// Stop and retry the conversation. If the error persists, open an issue in GitHub.
    Cancelled {
        /// The reason for the cancellation.
        reason: CancelledReason
    },
    /// The session started.
    SessionStarted {
        /// The session id.
        session_id: Uuid
    },
    /// The session stopped.
    SessionStopped {
        /// The session id.
        session_id: Uuid
    },
}

impl TryFrom<Message> for EventBase {
    type Error = Error;
    fn try_from(message: Message) -> Result<Self, Self::Error> {
        
        match message.path.as_str() { 
            "turn.start" => Err(Error::Skip),
            "turn.end" => Err(Error::Skip),
            _ => Err(Error::NoPath)
        }
        
        
        //todo!("Implement this function");
        // return match message {
        //     Message::Binary { .. } => Err(EventError::Unprocessable),
        //     Message::Text(text) => {
        //         
        //         //let (id, data, path) = extract_headers_and_data_from_text_message(text); 
        //         
        //         if data.is_none() {
        //             return Err(EventError::Unprocessable);
        //         }
        // 
        //         match path.as_str() {
        //             "turn.start" => Ok(EventBase::SessionStarted { session_id: id.parse().unwrap() }),
        //             "turn.end" => Ok(EventBase::SessionStopped { session_id: id.parse().unwrap() }),
        //             _ => Err(EventError::NoPath)
        //         }
        //     }
        // }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
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