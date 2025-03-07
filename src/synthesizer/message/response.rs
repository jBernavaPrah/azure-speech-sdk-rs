use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Audio {
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(rename = "streamId")]
    pub stream_id: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Context {
    #[serde(rename = "serviceTag")]
    pub service_tag: String,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Response {
    pub context: Option<Context>,
    pub audio: Audio,
}
