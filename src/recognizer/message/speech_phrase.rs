#![allow(dead_code)]

use crate::recognizer::message::common::{Language, RecognitionStatus};
use crate::recognizer::{Duration, Offset};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SpeechPhrase {
    #[serde(rename = "RecognitionStatus")]
    pub(crate) recognition_status: RecognitionStatus,
    #[serde(rename = "Offset")]
    pub(crate) offset: Option<Offset>,
    #[serde(rename = "Duration")]
    pub(crate) duration: Option<Duration>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SimpleSpeechPhrase {
    #[serde(rename = "DisplayText")]
    pub(crate) display_text: String,
    #[serde(rename = "PrimaryLanguage")]
    pub(crate) primary_language: Option<Language>,
    #[serde(rename = "SpeakerId")]
    pub(crate) speaker_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct DetailedSpeechPhrase {
    #[serde(rename = "NBest")]
    pub(crate) n_best: Vec<Phrase>,
    #[serde(rename = "PrimaryLanguage")]
    pub(crate) primary_language: Option<Language>,
    #[serde(rename = "DisplayText")]
    pub(crate) display_text: Option<String>,
    #[serde(rename = "SpeakerId")]
    pub(crate) speaker_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Phrase {
    #[serde(rename = "Confidence")]
    pub(crate) confidence: Option<String>,
    #[serde(rename = "Lexical")]
    pub(crate) lexical: String,
    #[serde(rename = "ITN")]
    pub(crate) itn: String,
    #[serde(rename = "MaskedITN")]
    pub(crate) masked_itn: String,
    #[serde(rename = "Display")]
    pub(crate) display: Option<String>,
    #[serde(rename = "DisplayText")]
    pub(crate) display_text: Option<String>,
    #[serde(rename = "Words")]
    pub(crate) words: Option<Vec<Word>>,
    #[serde(rename = "DisplayWords")]
    pub(crate) display_words: Option<Vec<Word>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Word {
    #[serde(rename = "Word")]
    pub(crate) word: String,
    #[serde(rename = "Offset")]
    pub(crate) offset: Offset,
    #[serde(rename = "Duration")]
    pub(crate) duration: Duration,
}
