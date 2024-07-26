use serde_json::{json, Value};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::connector::message::Message;
use crate::{Details, Spec};
use crate::config::Device;
use crate::recognizer::config::{RecognizerConfig};


/// Creates a speech configuration message.
///
/// # Arguments
///
/// * `session_id` - A Uuid representing the session id.
/// * `config` - A reference to the ResolverConfig struct which contains the configuration for the Azure STT service.
/// * `source` - A reference to the Source struct which contains the audio source details.
///
/// # Returns
///
/// * A Message struct containing the speech configuration message.
pub(crate) fn create_speech_config_message(session_id: Uuid,
                                           config: &RecognizerConfig,
                                           device: &Device,
                                           spec: &Spec,
                                           details: &Details,
) -> Message {

    let audio = json!({
        "source": {
            "connectivity": details.connectivity,
            "manufacturer": details.manufacturer,
            "model": details.model,
            "type": details.name,
            "samplerate": spec.sample_rate,
            "bitspersample": spec.bits_per_sample,
            "channelcount": spec.channels,
        }
    });

    

    Message::Text {
        id: session_id.to_string(),
        path: "speech.config".to_string(),
        headers: vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("X-Timestamp".to_string(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string()),
        ],
        data: Some(json!({
        "context": {
            "system": &device.system,
            "os": &device.os,
            "audio": audio.as_object().unwrap(),
        },
        "recognition": config.mode,
    }).to_string()),
    }
}

/// Creates a speech context message.
///
/// # Arguments
///
/// * `session_id` - A Uuid representing the session id.
/// * `config` - A reference to the ResolverConfig struct which contains the configuration for the Azure STT service.
///
/// # Returns
///
/// * A Message struct containing the speech context message.
pub(crate) fn create_speech_context_message(session_id: Uuid, config: &RecognizerConfig) -> Message {
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
            "languages": config.languages,
            "onSuccess": {
                "action": "Recognize"
            },
            "onUnknown": {
                "action": "None"
            }
        });

        let custom_models: Option<Value> = if let Some(custom_models) = config.custom_models.as_ref() {
            Some(custom_models.iter().map(|(l, e)| json!({
                "language": l,
                "endpoint": e,
            })).collect())
        } else { None };

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


    Message::Text {
        id: session_id.to_string(),
        path: "speech.context".to_string(),
        headers: vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("X-Timestamp".to_string(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string()),
        ],
        data: Some(context.to_string()),
    }
}

/// Creates a speech audio headers message.
///
/// # Arguments
///
/// * `session_id` - A Uuid representing the session id.
/// * `content_type` - A String representing the content type of the audio.
/// * `audio_headers` - A Headers struct containing the audio headers.
///
/// # Returns
///
/// * A Message struct containing the speech audio headers message.
pub(crate) fn create_speech_audio_headers_message(session_id: Uuid, content_type: &str, spec: &Spec) -> Message {
    Message::Binary {
        id: session_id.to_string(),
        path: "audio".to_string(),
        headers: vec![
            ("Content-Type".to_string(), content_type.to_string()),
            ("X-Timestamp".to_string(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string()),
        ],
        data: Some(spec.into_header_for_infinite_file()),
    }
}

/// Creates a speech audio message.
///
/// # Arguments
///
/// * `session_id` - A Uuid representing the session id.
/// * `data` - An Option containing a Vec<u8> which represents the audio data.
///
/// # Returns
///
/// * A Message struct containing the speech audio message.
pub(crate) fn create_speech_audio_message(session_id: Uuid, data: Option<Vec<u8>>) -> Message {
    Message::Binary {
        id: session_id.to_string(),
        path: "audio".to_string(),
        headers: vec![
            ("X-Timestamp".to_string(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string()),
        ],
        data,
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use hound::WavSpec;
    use crate::recognizer::config::{LanguageDetectMode, Os, OutputFormat, System};
    use crate::source::{Details};
    use serde_json::Value;
    use uuid::Uuid;

    #[test]
    fn test_create_speech_config_message() {
        let session_id = Uuid::new_v4();

        let message = create_speech_config_message(session_id,
                                                   &RecognizerConfig::default(),
                                                   &Device::default()
                                                       .set_os(Os::unknown())
                                                       .set_system(System::unknown()),
                                                   &WavSpec {
                                                       sample_format: hound::SampleFormat::Int,
                                                       bits_per_sample: 16,
                                                       channels: 1,
                                                       sample_rate: 16000,
                                                   }, &Details::unknown());
        match message {
            Message::Text { headers, data, id, path } => {
                assert_eq!(path, "speech.config");
                assert_eq!(id, session_id.to_string());
                assert_eq!(headers.len(), 2);
                assert_eq!(headers[0].0, "Content-Type");
                assert_eq!(headers[0].1, "application/json");
                assert_eq!(headers[1].0, "X-Timestamp");
                assert_eq!(data.is_some(), true);
                
                let json = serde_json::from_str::<Value>(data.unwrap().as_str()).unwrap();

                // test data
                assert_eq!(serde_json::from_str::<Value>(r#"{"context":{"audio":{"source":{"bitspersample":16,"channelcount":1,"connectivity":"Unknown","manufacturer":"Unknown","model":"Unknown","samplerate":16000,"type":"Unknown"}},"os":{"name":"Unknown","platform":"Unknown","version":"Unknown"},"system":{"build":"Unknown","lang":"Unknown","name":"Unknown","version":"Unknown"}},"recognition":"conversation"}"#).unwrap(), json);
            }
            _ => panic!("Expected Text message")
        }
    }

    #[test]
    fn test_create_speech_context_message() {
        let config = RecognizerConfig::default()
            .set_detect_languages(vec![String::from("en-us"), String::from("it-it")], LanguageDetectMode::Continuous)
            .set_output_format(OutputFormat::Detailed)
            .set_phrases(vec![String::from("hello world")])
            .set_custom_models(vec![("en-us".to_string(), "https://custom-model.com".to_string())]);

        let session_id = Uuid::new_v4();
        let message = create_speech_context_message(session_id, &config);
        match message {
            Message::Text { headers, data, id, path } => {
                assert_eq!(path, "speech.context");
                assert_eq!(id, session_id.to_string());

                assert_eq!(headers.len(), 2);
                assert_eq!(headers[0].0, "Content-Type");
                assert_eq!(headers[0].1, "application/json");
                assert_eq!(headers[1].0, "X-Timestamp");
                assert_eq!(data.is_some(), true);
                
                let json = serde_json::from_str::<Value>(data.unwrap().as_str()).unwrap();

                // test data
                assert_eq!(serde_json::from_str::<Value>(r#"{"dgi":{"Groups":[{"Items":[{"Text":"hello world"}],"Type":"Generic"}]},"languageId":{"Priority":"PrioritizeLatency","languages":["en-us","it-it"],"mode":"DetectContinuous","onSuccess":{"action":"Recognize"},"onUnknown":{"action":"None"}},"phraseDetection":{"customModels":[{"endpoint":"https://custom-model.com","language":"en-us"}],"onInterim":null,"onSuccess":null},"phraseOutput":{"interimResults":{"resultType":"Auto"},"phraseResults":{"resultType":"Always"}}}"#).unwrap(), json);
            }
            _ => panic!("Expected Text message")
        }
    }

    #[test]
    fn test_create_speech_audio_headers_message() {
        let session_id = Uuid::new_v4();

        let audio_headers = WavSpec {
            sample_format: hound::SampleFormat::Int,
            bits_per_sample: 16,
            channels: 1,
            sample_rate: 16000,

        };

        let message = create_speech_audio_headers_message(session_id,
                                                          "audio/x-wav",
                                                          &WavSpec {
                                                              sample_format: hound::SampleFormat::Int,
                                                              bits_per_sample: 16,
                                                              channels: 1,
                                                              sample_rate: 16000,
                                                          });
        match message {
            Message::Binary { headers, data, id, path } => {
                assert_eq!(path, "audio");
                assert_eq!(id, session_id.to_string());
                assert_eq!(headers.len(), 2);
                assert_eq!(headers[0].0, "Content-Type");
                assert_eq!(headers[0].1, "audio/x-wav");
                assert_eq!(headers[1].0, "X-Timestamp");
                assert_eq!(data.is_some(), true);

                // test data
                let audio_headers_vec: Vec<u8> = audio_headers.into_header_for_infinite_file();
                assert_eq!(data.unwrap(), audio_headers_vec);
            }
            _ => panic!("Expected Binary message")
        }
    }

    #[test]
    fn test_create_speech_audio_message() {
        let session_id = Uuid::new_v4();

        let audio = vec![0, 1, 2, 3, 4, 5];

        let message = crate::recognizer::utils::create_speech_audio_message(session_id, Some(audio.clone()));
        match message {
            crate::connector::message::Message::Binary { headers, data, id, path } => {
                assert_eq!(path, "audio");
                assert_eq!(id, session_id.to_string());
                assert_eq!(headers.len(), 1);
                assert_eq!(headers[0].0, "X-Timestamp");
                assert_eq!(data.is_some(), true);

                // test data
                assert_eq!(data.unwrap(), audio);
            }
            _ => panic!("Expected Binary message")
        }

        let message = crate::recognizer::utils::create_speech_audio_message(session_id, None);
        match message {
            crate::connector::message::Message::Binary { headers, data, id, path } => {
                assert_eq!(path, "audio");
                assert_eq!(id, session_id.to_string());

                assert_eq!(headers.len(), 1);
                assert_eq!(headers[0].0, "X-Timestamp");
                assert_eq!(data.is_some(), false);
            }
            _ => panic!("Expected Binary message")
        }
    }
}

