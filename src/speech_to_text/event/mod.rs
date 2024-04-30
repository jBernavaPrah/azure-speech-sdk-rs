pub mod cancelled;

use serde_json::Value;
use uuid::Uuid;
use crate::connector::message::Message;
use crate::speech_to_text::event::cancelled::{CancelCode, CancelReason};

pub trait TryFromMessage<T> {
    fn try_from_message(message: &Message) -> crate::errors::Result<Option<Event<T>>>;
}

#[derive(Debug)]
pub enum Event<T> {
    // try to remove the reason. It is not needed
    Cancelled { reason: CancelReason, code: CancelCode },
    SpeechStartDetected { offset: u32 },
    SpeechEndDetected { offset: u32 },
    SessionStarted { session_id: Uuid },
    SessionStopped { session_id: Uuid },
    R(T),
}

impl<T> TryFromMessage<T> for Event<T>
    where T: TryFromMessage<T> {
    fn try_from_message(message: &Message) -> crate::errors::Result<Option<Event<T>>> {

        if message.is_binary() {
            return Err(crate::errors::Error::new("Binary Message not expected"));
        }

        match message.path().as_ref().unwrap().as_str() {
            "speech.startdetected" => Ok(Some(Event::SpeechStartDetected {
                offset: message.json().unwrap().get("Offset").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
            })),
            "speech.enddetected" => Ok(Some(Event::SpeechEndDetected {
                offset: message.json().unwrap().get("Offset").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
            })),
            path => Err(crate::errors::Error::new(format!("Unmatched path {}", path))),
        }
    }
}

