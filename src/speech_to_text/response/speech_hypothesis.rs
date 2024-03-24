use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SpeechHypothesis {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "Text")]
    pub text: String,
    #[serde(rename = "Offset")]
    pub offset: u32,
    #[serde(rename = "Duration")]
    pub duration: u32,
    #[serde(rename = "PrimaryLanguage")]
    pub primary_language: PrimaryLanguage,
    #[serde(rename = "Channel")]
    pub channel: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrimaryLanguage {
    #[serde(rename = "Language")]
    pub language: String,
}