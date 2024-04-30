use std::fmt;

#[derive(Debug)]
pub enum CancelReason {
    /**
     * Indicates that an error occurred during speech recognition.
     */
    Error,

    /**
     * Indicates that the end of the audio stream was reached.
     */
    EndOfStream,
}

#[derive(Debug)]
pub enum CancelCode {
    /**
     * Indicates that no error occurred during speech recognition.
     */
    NoError,

    /**
     * Indicates an authentication error.
     */
    AuthenticationFailure,

    /**
     * Indicates that one or more recognition parameters are invalid.
     */
    BadRequestParameters,

    /**
     * Indicates that the number of parallel requests exceeded the number of allowed
     * concurrent transcriptions for the subscription.
     */
    TooManyRequests,

    /**
     * Indicates a connection error.
     */
    ConnectionFailure,

    /**
     * Indicates a time-out error when waiting for response from service.
     */
    ServiceTimeout,

    /**
     * Indicates that an error is returned by the service.
     */
    ServiceError,

    /**
     * Indicates an unexpected runtime error.
     */
    RuntimeError,

    /**
     * Indicates an quota overrun on existing key.
     */
    Forbidden,
}



impl fmt::Display for CancelCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CancelCode::Forbidden => write!(f, "The recognizer is using a free subscription that ran out of quota."),
            CancelCode::BadRequestParameters => write!(f, "Invalid parameter or unsupported audio format in the request."),
            CancelCode::TooManyRequests => write!(f, "The number of parallel requests exceeded the number of allowed concurrent transcriptions."),
            _ => write!(f, "The speech service encountered an internal error and could not continue."),
        }
    }
}