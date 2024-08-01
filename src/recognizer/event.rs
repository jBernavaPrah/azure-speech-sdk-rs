use std::fmt;
use serde::Deserialize;
use serde_json::Value;
use crate::connector::message::Message;
use crate::event::{EventError, CancelledReason};

#[derive(Debug, Clone)]
/// Event for the speech recognition
pub enum EventSpeech {
    /// The speech recognition started.
    StartDetected {
        /// The offset of the speech recognition. 
        /// The offset is the time in milliseconds from the start of the conversation.
        /// *Attention*: I'm not sure if this is the correct explanation.
        offset: u32
    },
    /// The speech recognition ended.
    EndDetected {
        /// The offset of the speech recognition.
        /// The offset is the time in milliseconds from the start of the conversation.
        /// *Attention*: I'm not sure if this is the correct explanation.
        offset: u32
    },
    /// Recognized event.
    /// Contains the recognized text, the offset, the duration, the primary language and the speaker id (if activated).
    Recognized {
        /// The recognized text.
        text: String,
        /// The offset of the recognized text.
        /// The offset is the time in milliseconds from the start of the conversation.
        /// *Attention*: I'm not sure if this is the correct explanation.
        offset: u32,
        /// The duration of the recognized text.
        /// The duration is in milliseconds.
        /// *Attention*: I'm not sure if this is correct.
        duration: u32,
        /// The primary language of the recognized text.
        primary_language: Option<PrimaryLanguage>,
        /// The speaker id of the recognized text.
        /// This will be None if the speaker id is not activated.
        speaker_id: Option<String>,
        /// The raw message.
        raw: Message,
    },
    /// Recognizing event.
    Recognizing {
        /// The recognized text.
        text: String,
        /// The offset of the recognized text.
        /// The offset is the time in milliseconds from the start of the conversation.
        /// *Attention*: I'm not sure if this is the correct explanation.
        offset: u32,
        /// The duration of the recognized text.
        /// The duration is in milliseconds.
        /// *Attention*: I'm not sure if this is correct.
        duration: u32,
        /// The primary language of the recognized text.
        primary_language: Option<PrimaryLanguage>,
        /// The speaker id of the recognized text.
        /// This will be None if the speaker id is not activated.
        speaker_id: Option<String>,
        /// The raw message.
        raw: Message,
    },
    /// UnMatch event.
    /// This event is triggered when the speech recognition does not match any text.
    UnMatch {
        /// The raw message.
        raw: Message
    },
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


impl TryFrom<Message> for EventSpeech {
    type Error = EventError;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        return match message {
            Message::Binary { .. } => Err(EventError::Skip),
            Message::Text { ref path, ref data, .. } => {
                if data.is_none() {
                    return Err(EventError::Unprocessable);
                }

                // todo: map error.
                let data = serde_json::from_str::<Value>(data.as_ref().unwrap().as_str()).unwrap();

                match path.as_str() {
                    "speech.startDetected" => Ok(EventSpeech::StartDetected {
                        offset: data.get("Offset").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
                    }),
                    "speech.endDetected" => Ok(EventSpeech::EndDetected {
                        offset: data.get("Offset").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
                    }),
                    "speech.hypothesis" | "speech.fragment" => Ok(EventSpeech::Recognizing {
                        text: data.get("Text").unwrap_or(&Value::String("".to_string())).as_str().unwrap().to_string(),
                        offset: data.get("Offset").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
                        duration: data.get("Duration").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
                        primary_language: PrimaryLanguage::try_from(&data).ok(),
                        speaker_id: data.get("SpeakerId").map(|x| x.as_str().unwrap().to_string()),
                        raw: message.clone(),
                    }),
                    "speech.phrase" => {
                        let status: RecognitionStatus = serde_json::from_value(data.get("RecognitionStatus")
                            .ok_or(EventError::Unprocessable)?.clone())
                            .map_err(|_| EventError::Unprocessable)?;

                        // Do nothing when the status is EndOfDictation,
                        // because it is already managed by base event, with
                        // the speech.endDetected event.
                        if status == RecognitionStatus::EndOfDictation {
                            return Err(EventError::Skip);
                        }

                        if status == RecognitionStatus::Success {
                            return Ok(EventSpeech::Recognized {
                                text: data.get("DisplayText").unwrap_or(&Value::String("".to_string())).as_str().unwrap().to_string(),
                                offset: data.get("Offset").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
                                duration: data.get("Duration").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
                                primary_language: PrimaryLanguage::try_from(&data).ok(),
                                speaker_id: data.get("SpeakerId").map(|x| x.as_str().unwrap_or("").to_string()),
                                raw: message.clone(),
                            });
                        }

                        if status == RecognitionStatus::NoMatch
                            || status == RecognitionStatus::InitialSilenceTimeout
                            || status == RecognitionStatus::BabbleTimeout
                        {
                            return Ok(EventSpeech::UnMatch { raw: message.clone() });
                        }

                        return Err(EventError::Error(CancelledReason::from(status)));
                    }
                    _ => Err(EventError::NoPath),
                }
            }
        };
    }
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub(crate) enum RecognitionStatus {
    Success,
    NoMatch,
    InitialSilenceTimeout,
    BabbleTimeout,
    Error,
    EndOfDictation,
    TooManyRequests,
    BadRequest,
    Forbidden,
}

impl From<RecognitionStatus> for CancelledReason {
    fn from(value: RecognitionStatus) -> Self {
        match value {
            RecognitionStatus::Error => CancelledReason::RuntimeError,
            RecognitionStatus::TooManyRequests => CancelledReason::TooManyRequests,
            RecognitionStatus::BadRequest => CancelledReason::RuntimeError,
            RecognitionStatus::Forbidden => CancelledReason::Forbidden,
            _ => unreachable!("This status is not an error.")
        }
    }
}


#[derive(Debug, Clone)]
/// Primary language
pub struct PrimaryLanguage {
    /// The language code
    pub language: String,
    /// The confidence of the language detection
    pub confidence: Option<String>,
}

impl PrimaryLanguage {
    pub(crate) fn new(language: impl Into<String>, confidence: Option<impl Into<String>>) -> Self {
        Self {
            language: language.into(),
            confidence: confidence.map(|x| x.into()),
        }
    }
}

impl TryFrom<&Value> for PrimaryLanguage {
    type Error = crate::Error;
    fn try_from(message: &Value) -> Result<Self, Self::Error> {
        let primary_language = message.get("PrimaryLanguage").ok_or(crate::Error::InvalidResponse("PrimaryLanguage not found in the response.".to_string()))?;

        Ok(PrimaryLanguage::new(
            primary_language.get("Language").unwrap_or(&Value::String("".to_string())).as_str().unwrap().to_string(),
            primary_language.get("Confidence").map(|x| x.as_str().unwrap().to_string()),
        ))
    }
}