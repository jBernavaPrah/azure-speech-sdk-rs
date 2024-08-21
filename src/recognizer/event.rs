use crate::recognizer::Language;
use crate::RequestId;

/// The raw text of message.
///
/// The raw message is the message received from the speech recognition service.
pub type RawMessage = String;

/// Recognizer events.
///
/// The events are used to notify the user of the progress of the speech recognition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// The session started.
    SessionStarted(RequestId),

    /// The session ended.
    SessionEnded(RequestId),

    /// The speech recognition started.
    StartDetected(RequestId, Offset),
    /// The speech recognition ended.
    EndDetected(RequestId, Offset),

    /// Recognizing event.
    Recognizing(RequestId, Recognized, Offset, Duration, RawMessage),

    /// Recognized event.
    Recognized(RequestId, Recognized, Offset, Duration, RawMessage),

    /// UnMatch event.
    /// This event is triggered when the speech recognition does not match any text.
    UnMatch(RequestId, Offset, Duration, RawMessage),
    //Cancelled(RequestId, Offset, crate::Error),
}

/// The offset of the speech recognition.
///
/// The offset is the time in milliseconds from the start of the conversation.
pub type Offset = u64;

/// The duration of the speech recognition.
///
/// The duration is the time in milliseconds of the speech recognition.
pub type Duration = u64;

/// The recognized text.
///
/// Contains the recognized text, the primary language and the speaker id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recognized {
    /// The recognized text.
    pub text: String,
    /// The primary language of the recognized text.
    pub primary_language: Option<PrimaryLanguage>,
    
    // todo: Remove from here and add to a diarization module.
    /// The speaker id of the recognized text.
    /// This will be None if the detection of the speaker is not activated.
    pub speaker_id: Option<String>,
}

/// The confidence of the speech recognition.
///
/// The confidence is the confidence of the speech recognition.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Confidence {
    Low,
    Normal,
    High,
    #[default]
    Unknown,
}

impl From<&str> for Confidence {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl From<String> for Confidence {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "low" => Confidence::Low,
            "normal" => Confidence::Normal,
            "high" => Confidence::High,
            _ => Confidence::Unknown,
        }
    }
}

/// Primary language
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimaryLanguage {
    /// The language code
    pub language: Language,
    /// The confidence of the language detection
    pub confidence: Confidence,
}

impl PrimaryLanguage {
    pub(crate) fn new(language: Language, confidence: Confidence) -> Self {
        Self {
            language,
            confidence,
        }
    }
}
