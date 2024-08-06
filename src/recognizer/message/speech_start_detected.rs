use crate::recognizer::Offset;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Default)]
pub(crate) struct SpeechStartDetected {
    #[serde(rename = "Offset")]
    pub(crate) offset: Offset,
}
