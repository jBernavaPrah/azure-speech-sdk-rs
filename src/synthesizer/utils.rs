use crate::connector::make_text_payload;
use crate::synthesizer::config::Config;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_websockets::Message;

/// Creates a speech configuration message.
pub(crate) fn create_speech_config_message(request_id: String, config: &Config) -> Message {
    Message::text(make_text_payload(
        vec![
            ("X-RequestId".to_string(), request_id),
            ("Path".to_string(), "speech.config".to_string()),
            ("Content-Type".to_string(), "application/json".to_string()),
            (
                "X-Timestamp".to_string(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    .to_string(),
            ),
        ],
        Some(
            &json!({"context":{"system":&config.device.system,"os":&config.device.os}}).to_string(),
        ),
    ))
}

/// Creates a speech context message.
pub(crate) fn create_synthesis_context_message(request_id: String, config: &Config) -> Message {
    Message::text(make_text_payload(
        vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            (
                "X-Timestamp".to_string(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    .to_string(),
            ),
            ("X-RequestId".to_string(), request_id),
            ("Path".to_string(), "synthesis.context".to_string()),
        ],
        Some(
            &json!({"synthesis":
            {"audio":
                {"metadataOptions":
                    {
                        "bookmarkEnabled": config.bookmark_enabled,
                        "punctuationBoundaryEnabled": config.punctuation_boundary_enabled,
                        "sentenceBoundaryEnabled": config.sentence_boundary_enabled,
                        "sessionEndEnabled": config.session_end_enabled,
                        "visemeEnabled": config.viseme_enabled,
                        "wordBoundaryEnabled": config.word_boundary_enabled
                    },
                    "outputFormat": config.audio_format.as_str()
                },
                "language": {"autoDetection": config.auto_detect_language}
            }})
            .to_string(),
        ),
    ))
}

pub(crate) fn create_ssml_message(request_id: String, ssml: &str) -> Message {
    Message::text(make_text_payload(
        vec![
            (
                "Content-Type".to_string(),
                "application/ssml+xml".to_string(),
            ),
            (
                "X-Timestamp".to_string(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    .to_string(),
            ),
            ("X-RequestId".to_string(), request_id),
            ("Path".to_string(), "ssml".to_string()),
        ],
        Some(ssml),
    ))
}
