use serde::Deserialize;
use serde_json::Value;
use crate::connector::message::Message;
use crate::speech_to_text::event::cancelled::{CancelCode, CancelReason};
use crate::speech_to_text::event::TryFromMessage;
use crate::speech_to_text::event::Event as BaseEvent;


#[derive(Debug)]
pub enum Event {
    Recognized {
        text: String,
        offset: u32,
        duration: u32,
        primary_language: PrimaryLanguage,
        speaker_id: Option<String>,

        raw: Message,
    },
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

impl TryFromMessage<Event> for Event {
    fn try_from_message(message: &Message) -> crate::errors::Result<Option<BaseEvent<Event>>> {

        // try to convert the message also to a basic event
        let event = BaseEvent::try_from_message(&message);
        if event.is_ok() {
            return Ok(event.unwrap());
        }

        if message.is_binary() {
            return Err(crate::errors::Error::new("Binary Message not expected"));
        }

        match message.path().as_ref().unwrap().as_str() {
            //  case "speech.hypothesis":
            //             case "speech.fragment":

            "speech.hypothesis" | "speech.fragment" => Ok(Some(BaseEvent::R(Event::Recognizing {
                text: message.json().unwrap().get("Text").unwrap_or(&Value::String("".to_string())).as_str().unwrap().to_string(),
                offset: message.json().unwrap().get("Offset").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
                duration: message.json().unwrap().get("Duration").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
                primary_language: PrimaryLanguage {
                    language: message.json().unwrap().get("PrimaryLanguage").unwrap().get("Language").unwrap_or(&Value::String("".to_string())).as_str().unwrap().to_string(),
                    confidence: message.json().unwrap().get("PrimaryLanguage").unwrap().get("Confidence").map(|x| x.as_str().unwrap().to_string()),
                },
                speaker_id: None,
                raw: message.clone(),
            }))),
            "speech.phrase" => {
                let status: RecognitionStatus = serde_json::from_value(message.json().unwrap().get("RecognitionStatus").unwrap().clone()).unwrap();

                // Do nothing when the status is EndOfDictation,
                // because it is already managed by base event, with 
                // the speech.endDetected event.
                if status == RecognitionStatus::EndOfDictation {
                    return Ok(None);
                }
                
                if status == RecognitionStatus::Success {
                    return Ok(Some(BaseEvent::R(Event::Recognized {
                        text: message.json().unwrap().get("DisplayText").unwrap_or(&Value::String("".to_string())).as_str().unwrap().to_string(),
                        offset: message.json().unwrap().get("Offset").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
                        duration: message.json().unwrap().get("Duration").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
                        primary_language: PrimaryLanguage {
                            language: message.json().unwrap().get("PrimaryLanguage").unwrap().get("Language").unwrap_or(&Value::String("".to_string())).as_str().unwrap().to_string(),
                            confidence: message.json().unwrap().get("PrimaryLanguage").unwrap().get("Confidence").map(|x| x.as_str().unwrap().to_string()),
                        },
                        speaker_id: message.json().unwrap().get("SpeakerId").map(|x| x.as_str().unwrap().to_string()),
                        raw: message.clone(),
                    })));
                }

                if status == RecognitionStatus::NoMatch
                    || status == RecognitionStatus::InitialSilenceTimeout
                    || status == RecognitionStatus::BabbleTimeout
                {
                    return Ok(Some(BaseEvent::R(Event::UnMatch { raw: message.clone() })));
                }

                // todo: check all the errors and match correctly here.
                Ok(Some(BaseEvent::Cancelled {
                    reason: CancelReason::Error,
                    code: CancelCode::ServiceError,
                }))
            }
            path => Err(crate::errors::Error::new(format!("Unmatched path {}", path))),
        }
    }
}


#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub enum RecognitionStatus {
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
pub struct PrimaryLanguage {
    pub language: String,
    pub confidence: Option<String>,
}