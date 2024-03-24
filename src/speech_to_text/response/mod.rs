use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;
pub use crate::speech_to_text::response::speech_end::SpeechEnd;
pub use crate::speech_to_text::response::speech_hypothesis::{PrimaryLanguage, SpeechHypothesis};
pub use crate::speech_to_text::response::speech_phrase::{Words, NBest, RecognitionStatus, SpeechPhrase};
pub use crate::speech_to_text::response::speech_start::{SpeechStart};
pub use crate::speech_to_text::response::turn_start::{Context, TurnStart};

pub mod turn_start;
pub mod speech_start;
pub mod speech_hypothesis;
pub mod speech_phrase;
pub mod speech_end;

#[derive(Debug, Clone, Deserialize)]
pub enum Response {
    TurnStart(TurnStart),
    SpeechStart(SpeechStart),
    SpeechHypothesis(SpeechHypothesis),
    SpeechPhrase(SpeechPhrase),
    SpeechEnd(SpeechEnd),
    TurnEnd,
    UnknownPath{ path: String, json: Value },
    ErrorDecoding { path: String, json: Value },
}



impl Response {
    pub(crate) fn from_message(message: Message) -> Self {
        let binding = message.clone().to_string();
        let mut split_response = binding.split("\r\n\r\n");

        let headers: Vec<(String, String)> = split_response.nth(0).unwrap()
            .split("\r\n")
            .map(|x| {
                let mut split = x.split(":");
                (split.nth(0).unwrap().to_string(), split.nth(0).unwrap().to_string())
            })
            .collect();

        let json = split_response.nth(0).unwrap();

        let path = match headers.iter().find(|x| x.0 == "Path") {
            Some(p) => p.1.clone(),
            None => return Response::ErrorDecoding { path: String::from(""), json: serde_json::from_str::<Value>(json).unwrap() }
        };

         return match path.as_str() {
            "turn.start" => {
                if let Some(r) = serde_json::from_str::<TurnStart>(json).ok() {
                    return Response::TurnStart(r);
                }
                Response::ErrorDecoding { path: String::from("turn.start"), json: serde_json::from_str::<Value>(json).unwrap() }
            }
            "speech.startDetected" => {
                if let Some(r) = serde_json::from_str::<SpeechStart>(json).ok() {
                    return Response::SpeechStart(r);
                }
                Response::ErrorDecoding { path: String::from("speech.startDetected"), json: serde_json::from_str::<Value>(json).unwrap() }
            }
            "speech.hypothesis" => {
                if let Some(r) = serde_json::from_str::<SpeechHypothesis>(json).ok() {
                    return Response::SpeechHypothesis(r);
                }
                Response::ErrorDecoding { path: String::from("speech.hypothesis"), json: serde_json::from_str::<Value>(json).unwrap() }
            }
            "speech.phrase" => {
                if let Some(r) = serde_json::from_str::<SpeechPhrase>(json).ok() {
                    return Response::SpeechPhrase(r);
                }
                Response::ErrorDecoding { path: String::from("speech.phrase"), json: serde_json::from_str::<Value>(json).unwrap() }
            }
            "speech.endDetected" => {
                if let Some(r) = serde_json::from_str::<SpeechEnd>(json).ok() {
                    return Response::SpeechEnd(r);
                }
                Response::ErrorDecoding { path: String::from("speech.endDetected"), json: serde_json::from_str::<Value>(json).unwrap() }
            }
            "turn.end" => {
                Response::TurnEnd
            }
             e => {
                 Response::UnknownPath { path: String::from(e), json: serde_json::from_str::<Value>(json).unwrap() }
             }
         }
    }
}