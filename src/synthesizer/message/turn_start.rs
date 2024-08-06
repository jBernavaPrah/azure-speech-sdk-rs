use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct Webrtc {
    #[serde(rename = "connectionString")]
    pub connection_string: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct TurnStart {
    pub webrtc: Option<Webrtc>,
}
