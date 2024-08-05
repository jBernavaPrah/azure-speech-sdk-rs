use serde::Deserialize;
use crate::recognizer::Offset;

#[derive(Deserialize, Debug, Clone, Default)]
pub(crate) struct SpeechStartDetected {
    #[serde(rename = "Offset")]
    pub(crate) offset: Offset,
}