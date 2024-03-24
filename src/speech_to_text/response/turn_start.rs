use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnStart {
    pub context: Context,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    #[serde(rename = "serviceTag")]
    pub service_tag: String,
}
