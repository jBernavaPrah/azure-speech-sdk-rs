use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct SpeechEnd {
    #[serde(rename = "Offset")]
    pub offset: u32,
}