use serde::Deserialize;
use serde_json::Value;
use crate::connector::message::Message;
use crate::recognizer::event::{CancelledReason, EventBase, EventError, FromMessage};
use crate::recognizer::event::Event;


#[derive(Debug)]
/// Event for the speech recognition
pub enum EventSpeech {
    /// Recognized event. 
    /// Contains the recognized text, the offset, the duration, the primary language and the speaker id (if activated).
    Recognized {
        text: String,
        offset: u32,
        duration: u32,
        primary_language: PrimaryLanguage,
        speaker_id: Option<String>,

        raw: Message,
    },
    /// Recognizing event.
    Recognizing {
        text: String,
        offset: u32,
        duration: u32,
        primary_language: PrimaryLanguage,
        speaker_id: Option<String>,

        raw: Message,
    },
    UnMatch { raw: Message },
}

impl FromMessage<EventSpeech> for EventSpeech {
    fn from_message(message: &Message) -> Result<Event<EventSpeech>, EventError> {
        if message.is_binary() || message.json().is_none() {
            return Err(EventError::Unprocessable);
        }

        let json_message = message.json().unwrap();

        match message.path().as_ref().unwrap().as_str() {
            "speech.hypothesis" | "speech.fragment" => Ok(Event::Specific(EventSpeech::Recognizing {
                text: json_message.get("Text").unwrap_or(&Value::String("".to_string())).as_str().unwrap().to_string(),
                offset: json_message.get("Offset").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
                duration: json_message.get("Duration").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
                primary_language: PrimaryLanguage::from(&json_message),
                speaker_id: json_message.get("SpeakerId").map(|x| x.as_str().unwrap().to_string()),
                raw: message.clone(),
            })),
            "speech.phrase" => {
                let status: RecognitionStatus = serde_json::from_value(json_message.get("RecognitionStatus").unwrap().clone()).unwrap();

                // Do nothing when the status is EndOfDictation,
                // because it is already managed by base event, with 
                // the speech.endDetected event.
                if status == RecognitionStatus::EndOfDictation {
                    return Err(EventError::Skip);
                }

                if status == RecognitionStatus::Success {
                    return Ok(Event::Specific(EventSpeech::Recognized {
                        text: json_message.get("DisplayText").unwrap_or(&Value::String("".to_string())).as_str().unwrap().to_string(),
                        offset: json_message.get("Offset").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
                        duration: json_message.get("Duration").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
                        primary_language: PrimaryLanguage::from(&json_message),
                        speaker_id: json_message.get("SpeakerId").map(|x| x.as_str().unwrap().to_string()),
                        raw: message.clone(),
                    }));
                }

                if status == RecognitionStatus::NoMatch
                    || status == RecognitionStatus::InitialSilenceTimeout
                    || status == RecognitionStatus::BabbleTimeout
                {
                    return Ok(Event::Specific(EventSpeech::UnMatch { raw: message.clone() }));
                }

                // todo: check all the errors and match correctly here.
                Ok(Event::Base(EventBase::Cancelled {
                    reason: CancelledReason::ServiceError,
                }))
            }
            _ => Err(EventError::NoPath),
        }
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


#[derive(Debug)]
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

impl From<&Value> for PrimaryLanguage {
    fn from(message: &Value) -> Self {
        let primary_language = message.get("PrimaryLanguage").expect("PrimaryLanguage not found in the response.");

        PrimaryLanguage::new(
            primary_language.get("Language").unwrap_or(&Value::String("".to_string())).as_str().unwrap().to_string(),
            primary_language.get("Confidence").map(|x| x.as_str().unwrap().to_string()),
        )
    }
}