use crate::recognizer::config::Config;
use crate::recognizer::{AudioDevice, AudioFormat};
use crate::{make_binary_payload, make_text_payload};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_websockets::Message;

pub(crate) fn create_speech_config_message(
    request_id: String,
    config: &Config,
    audio_device: &AudioDevice,
) -> Message {
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
            &json!({
                "context": {
                    "system": config.device.system,
                    "os": config.device.os,
                    "audio": {
                        "source": {
                            "connectivity": audio_device.connectivity,
                            "manufacturer": audio_device.manufacturer,
                            "model": audio_device.model,
                            "type": audio_device.name,
                            //"samplerate": spec.sample_rate,
                            //"bitspersample": spec.bits_per_sample,
                            //"channelcount": spec.channels,
                        }
                    },
                },
                "recognition": config.mode,
            })
            .to_string(),
        ),
    ))
}

pub(crate) fn create_speech_context_message(request_id: String, config: &Config) -> Message {
    let mut context = json!({});

    if let Some(grammars) = config.phrases.as_ref() {
        let texts: Vec<Value> = grammars.iter().map(|x| json!({ "Text": x })).collect();

        context["dgi"] = json!({
            "Groups": [
                {
                    "Type": "Generic",
                    "Items": texts,
                }
            ]
        });
    }

    if config.languages.len() > 1 {
        context["languageId"] = json!({
            "mode": config.language_detect_mode.as_ref().unwrap(),
            "Priority": "PrioritizeLatency",
            "languages": config.languages.iter().map(|x| x.to_string()).collect::<Vec<String>>(),
            "onSuccess": {
                "action": "Recognize"
            },
            "onUnknown": {
                "action": "None"
            }
        });

        let custom_models: Option<Value> = config.custom_models.as_ref().map(|custom_models| {
            custom_models
                .iter()
                .map(|(l, e)| {
                    json!({
                        "language": l,
                        "endpoint": e,
                    })
                })
                .collect()
        });

        context["phraseDetection"] = json!({

            // "mode": "Conversation",
            // "speakerDiarization": {
            //     "mode": "Anonymous",
            //     "audioSessionId": "1",
            //     "audioOffsetMs": 0
            // },

            "customModels": custom_models,
            // todo: when translation, this are set to { action: "Translate" }
            "onInterim": Value::Null,
            // todo: when translation, this are set to { action: "Translate" }
            "onSuccess": Value::Null,
        });

        context["phraseOutput"] = json!({
            "interimResults": {
                "resultType": "Auto"
            },
            "phraseResults": {
                "resultType": "Always"
            }
        });
    }

    Message::text(make_text_payload(
        vec![
            ("X-RequestId".to_string(), request_id.to_string()),
            ("Path".to_string(), "speech.context".to_string()),
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
        Some(&context.to_string()),
    ))
}

pub(crate) fn create_audio_header_message(
    request_id: String,
    content_type: AudioFormat,
) -> Message {
    let mut headers = vec![
        ("Path".to_string(), "audio".to_string()),
        ("X-RequestId".to_string(), request_id),
        (
            "X-Timestamp".to_string(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .to_string(),
        ),
    ];

    headers.push((
        "Content-Type".to_string(),
        content_type.as_content_type().to_string(),
    ));

    Message::binary(make_binary_payload(
        headers,
        content_type.as_header().as_deref(),
    ))
}

pub(crate) fn create_audio_message(request_id: String, data: Option<&[u8]>) -> Message {
    let headers = vec![
        ("Path".to_string(), "audio".to_string()),
        ("X-RequestId".to_string(), request_id),
        (
            "X-Timestamp".to_string(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .to_string(),
        ),
    ];

    Message::binary(make_binary_payload(headers, data))
}
