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
    Recognizing(RequestId, Offset, Duration, Recognized, RawMessage),

    /// Recognized event.
    /// Contains the recognized text, the offset, the duration, the primary language and the speaker id (if activated).
    Recognized(RequestId, Offset, Duration, Recognized, RawMessage),

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
    pub primary_language: Option<Language>,
    /// The speaker id of the recognized text.
    /// This will be None if the detection of the speaker is not activated.
    pub speaker_id: Option<String>,
}


// 
// 
// impl TryFrom<Message> for EventSpeech {
//     type Error = EventError;
// 
//     fn try_from(message: Message) -> Result<Self, Self::Error> {
//         return match message {
//             Message::Binary { .. } => Err(EventError::Skip),
//             Message::Text { ref path, ref data, .. } => {
//                 if data.is_none() {
//                     return Err(EventError::Unprocessable);
//                 }
// 
//                 // todo: map error.
//                 let data = serde_json::from_str::<Value>(data.as_ref().unwrap().as_str()).unwrap();
// 
//                 match path.as_str() {
//                     "speech.startDetected" => Ok(EventSpeech::StartDetected {
//                         offset: data.get("Offset").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
//                     }),
//                     "speech.endDetected" => Ok(EventSpeech::EndDetected {
//                         offset: data.get("Offset").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
//                     }),
//                     "speech.hypothesis" | "speech.fragment" => Ok(EventSpeech::Recognizing {
//                         text: data.get("Text").unwrap_or(&Value::String("".to_string())).as_str().unwrap().to_string(),
//                         offset: data.get("Offset").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
//                         duration: data.get("Duration").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
//                         primary_language: PrimaryLanguage::try_from(&data).ok(),
//                         speaker_id: data.get("SpeakerId").map(|x| x.as_str().unwrap().to_string()),
//                         raw: message.clone(),
//                     }),
//                     "speech.phrase" => {
//                         let status: RecognitionStatus = serde_json::from_value(data.get("RecognitionStatus")
//                             .ok_or(EventError::Unprocessable)?.clone())
//                             .map_err(|_| EventError::Unprocessable)?;
// 
//                         // Do nothing when the status is EndOfDictation,
//                         // because it is already managed by base event, with
//                         // the speech.endDetected event.
//                         if status == RecognitionStatus::EndOfDictation {
//                             return Err(EventError::Skip);
//                         }
// 
//                         if status == RecognitionStatus::Success {
//                             return Ok(EventSpeech::Recognized {
//                                 text: data.get("DisplayText").unwrap_or(&Value::String("".to_string())).as_str().unwrap().to_string(),
//                                 offset: data.get("Offset").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
//                                 duration: data.get("Duration").unwrap_or(&Value::Number(0.into())).as_u64().unwrap() as u32,
//                                 primary_language: PrimaryLanguage::try_from(&data).ok(),
//                                 speaker_id: data.get("SpeakerId").map(|x| x.as_str().unwrap_or("").to_string()),
//                                 raw: message.clone(),
//                             });
//                         }
// 
//                         if status == RecognitionStatus::NoMatch
//                             || status == RecognitionStatus::InitialSilenceTimeout
//                             || status == RecognitionStatus::BabbleTimeout
//                         {
//                             return Ok(EventSpeech::UnMatch { raw: message.clone() });
//                         }
// 
//                         return Err(EventError::Error(CancelledReason::from(status)));
//                     }
//                     _ => Err(EventError::NoPath),
//                 }
//             }
//         };
//     }
// }


// impl From<RecognitionStatus> for crate::Error {
//     fn from(value: RecognitionStatus) -> Self {
//         match value {
//             RecognitionStatus::Error => crate::Error::RuntimeError("An error occurred during the recognition.".to_string()),
//             RecognitionStatus::TooManyRequests => crate::Error::TooManyRequests,
//             RecognitionStatus::BadRequest => crate::Error::RuntimeError("The request is invalid.".to_string()),
//             RecognitionStatus::Forbidden => crate::Error::Forbidden,
//             _ => unreachable!("This status is not an error.")
//         }
//     }
// }


#[derive(Debug, Clone, PartialEq, Eq)]
/// Primary language
pub struct Language {
    /// The language code
    pub language: String,
    /// The confidence of the language detection
    pub confidence: Option<String>,
}

impl Language {
    pub(crate) fn new(language: impl Into<String>, confidence: Option<impl Into<String>>) -> Self {
        Self {
            language: language.into(),
            confidence: confidence.map(|x| x.into()),
        }
    }
}