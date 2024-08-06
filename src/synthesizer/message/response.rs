use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct Audio {
    #[serde(rename = "type")]
    pub(crate) r#type: String,
    #[serde(rename = "streamId")]
    pub(crate) stream_id: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct Context {
    #[serde(rename = "serviceTag")]
    pub(crate) service_tag: String,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Response {
    pub(crate) context: Option<Context>,
    pub(crate) audio: Audio,
}
