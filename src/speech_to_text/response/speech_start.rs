use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechStart {
    #[serde(rename = "Offset")]
    pub offset: u32,
}


