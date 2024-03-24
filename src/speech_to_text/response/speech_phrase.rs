use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SpeechPhrase {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "RecognitionStatus")]
    pub recognition_status: RecognitionStatus,
    #[serde(rename = "Offset")]
    pub offset: u32,
    #[serde(rename = "Duration")]
    pub duration: u32,
    #[serde(rename = "Channel")]
    pub channel: u32,
    #[serde(rename = "DisplayText")]
    pub display_text: Option<String>,
    #[serde(rename = "NBest")]
    pub n_best: Option<Vec<NBest>>,
    #[serde(rename = "PrimaryLanguage")]
    pub primary_language: Option<PrimaryLanguage>

}

#[derive(Debug, Clone, Deserialize)]
pub struct PrimaryLanguage {
    #[serde(rename = "Language")]
    pub language: String,
    #[serde(rename = "Confidence")]
    pub confidence: String
}

#[derive(Debug, Clone, Deserialize)]
pub enum RecognitionStatus {
    Success,
    NoMatch,
    InitialSilenceTimeout,
    BabbleTimeout,
    Error,
    EndOfDictation,
    TooManyRequests,
    BadRequest,
    Forbidden,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Words {
    #[serde(rename = "Word")]
    pub word: String,
    #[serde(rename = "Offset")]
    pub offset: u32,
    #[serde(rename = "Duration")]
    pub duration: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NBest {
    #[serde(rename = "Confidence")]
    pub confidence: f32,
    #[serde(rename = "Lexical")]
    pub lexical: String,
    #[serde(rename = "ITN")]
    pub itn: String,
    #[serde(rename = "MaskedITN")]
    pub masked_itn: String,
    #[serde(rename = "Display")]
    pub display: String,
    #[serde(rename = "Words")]
    pub words: Option<Vec<Words>>,
}