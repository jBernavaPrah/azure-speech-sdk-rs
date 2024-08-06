use crate::recognizer::Offset;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Default)]
pub(crate) struct SpeechEndDetected {
    #[serde(rename = "Offset")]
    pub(crate) offset: Offset,
}
