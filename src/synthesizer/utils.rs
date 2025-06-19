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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{synthesizer::Config, Data, Message as EzMessage};

    #[test]
    fn test_create_speech_config_message() {
        let config = Config::new();
        let ws_msg = create_speech_config_message("id".to_string(), &config);
        let msg = EzMessage::try_from(ws_msg).unwrap();

        assert_eq!(msg.path, "speech.config");
        assert_eq!(msg.id, "id");
        assert_eq!(msg.get_header("Content-Type").unwrap(), "application/json");
        assert!(msg.get_header("X-Timestamp").is_some());

        match msg.data {
            Data::Text(Some(ref body)) => {
                let v: serde_json::Value = serde_json::from_str(body).unwrap();
                assert!(v.get("context").is_some());
            }
            _ => panic!("expected text body"),
        }
    }

    #[test]
    fn test_create_ssml_message() {
        let ws_msg = create_ssml_message("id".to_string(), "<speak>Hello</speak>");
        let msg = EzMessage::try_from(ws_msg).unwrap();

        assert_eq!(msg.path, "ssml");
        assert_eq!(msg.id, "id");
        assert_eq!(
            msg.get_header("Content-Type").unwrap(),
            "application/ssml+xml"
        );
        assert!(matches!(msg.data, Data::Text(Some(_))));
    }
}
