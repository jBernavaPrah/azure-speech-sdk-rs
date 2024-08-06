use crate::recognizer::message::common::Language;
use crate::recognizer::{Duration, Offset};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct SpeechHypothesis {
    #[serde(rename = "Text")]
    pub(crate) text: String,
    #[serde(rename = "Offset")]
    pub(crate) offset: Offset,
    #[serde(rename = "Duration")]
    pub(crate) duration: Duration,
    #[serde(rename = "PrimaryLanguage")]
    pub(crate) primary_language: Option<Language>,
    #[serde(rename = "SpeakerId")]
    pub(crate) speaker_id: Option<String>,
}
