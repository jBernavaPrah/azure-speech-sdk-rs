use crate::RequestId;

pub type RawMessage = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    SessionStarted(RequestId),
    SessionEnded(RequestId),

    /// The speech recognition started.
    StartDetected(RequestId, Offset),
    /// The speech recognition ended.
    EndDetected(RequestId, Offset),

    /// Recognizing event.
    Recognizing(RequestId, Recognized, Offset, Duration, RawMessage),

    /// Recognized event.
    /// Contains the recognized text, the offset, the duration, the primary language and the speaker id (if activated).
    Recognized(RequestId, Recognized, Offset, Duration, RawMessage),

    /// UnMatch event.
    /// This event is triggered when the speech recognition does not match any text.
    UnMatch(RequestId, Offset, Duration, RawMessage),
    //Cancelled(RequestId, Offset, crate::Error),
}

/// The offset of the speech recognition.
/// The offset is the time in milliseconds from the start of the conversation.
pub type Offset = u64;
pub type Duration = u64;

pub type Confidence = f64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recognized {
    /// The recognized text.
    pub text: String,
    /// The primary language of the recognized text.
    pub primary_language: Option<PrimaryLanguage>,
    /// The speaker id of the recognized text.
    /// This will be None if the detection of the speaker is not activated.
    pub speaker_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
