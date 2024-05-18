use serde_json::{json, Value};
use url::Url;
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::connector::message::Message;
use crate::recognizer::config::{OutputFormat, ResolverConfig};
use crate::recognizer::{Source, WavSpec};
use crate::recognizer::source::Sample;
use crate::utils::get_azure_hostname_from_region;

/// Generates the URI for Azure's Speech to Text service.
///
/// # Arguments
///
/// * `config` - A reference to the ResolverConfig struct which contains the configuration for the Azure STT service.
///
/// # Returns
///
/// * A String containing the generated URI.
pub(crate) fn generate_uri_for_stt_speech_azure(config: &ResolverConfig) -> String {
    let mut url = Url::parse(format!("wss://{}.stt.speech{}", config.auth.region, get_azure_hostname_from_region(&config.auth.region)).as_str()).unwrap();

    url.set_path(config.mode.to_uri_path().as_str());

    let lang = config.languages.first().expect("At least one language!");

    url.query_pairs_mut().append_pair("Ocp-Apim-Subscription-Key", config.auth.subscription.to_string().as_str());
    url.query_pairs_mut().append_pair("language", lang.as_str());
    url.query_pairs_mut().append_pair("format", &config.output_format.as_str());
    url.query_pairs_mut().append_pair("profanity", &config.profanity.as_str());
    url.query_pairs_mut().append_pair("storeAudio", &config.store_audio.to_string());

    if config.output_format == OutputFormat::Detailed {
        url.query_pairs_mut().append_pair("wordLevelTimestamps", "true");
    }

    if config.languages.len() > 1 {
        url.query_pairs_mut().append_pair("lidEnabled", true.to_string().as_str());
    }

    if let Some(ref connection_id) = config.connection_id {
        url.query_pairs_mut().append_pair("X-ConnectionId", connection_id.as_str());
    }

    url.to_string()
}


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
pub(crate) fn create_speech_config_message<T:Sample>(session_id: Uuid, config: &ResolverConfig, source: &Source<T>) -> Message {
    let system = json!({
        "name": config.system.name,
        "version": config.system.version,
        "build": config.system.build,
        "lang": config.system.lang,
    });

    let os = json!({
        "platform": config.os.platform,
        "name": config.os.name,
        "version": config.os.version,
    });

    let audio = json!({
        "source": {
            "connectivity": source.details.connectivity,
            "manufacturer": source.details.manufacturer,
            "model": source.details.model,
            "type": source.details.name,
            "samplerate": source.spec.sample_rate,
            "bitspersample": source.spec.bits_per_sample,
            "channelcount": source.spec.channels,
        }
    });


    Message::Text {
        headers: vec![
            ("Path".to_string(), "speech.config".to_string()),
            ("X-RequestId".to_string(), session_id.to_string()),
            ("Content-Type".to_string(), "application/json".to_string()),
            ("X-Timestamp".to_string(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string()),
        ],
        data: Some(json!({
        "context": {
            "system": system.as_object().unwrap(),
            "os": os.as_object().unwrap(),
            "audio": audio.as_object().unwrap(),
        },
        "recognition": config.mode,
    })),
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
pub(crate) fn create_speech_context_message(session_id: Uuid, config: &ResolverConfig) -> Message {
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
        headers: vec![
            ("Path".to_string(), "speech.context".to_string()),
            ("X-RequestId".to_string(), session_id.to_string()),
            ("Content-Type".to_string(), "application/json".to_string()),
            ("X-Timestamp".to_string(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string()),
        ],
        data: Some(context),
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
pub(crate) fn create_speech_audio_headers_message(session_id: Uuid, content_type: String, audio_headers: WavSpec) -> Message {
    Message::Binary {
        headers: vec![
            ("Path".to_string(), "audio".to_string()),
            ("X-RequestId".to_string(), session_id.to_string()),
            ("Content-Type".to_string(), content_type),
            ("X-Timestamp".to_string(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string()),
        ],
        data: Some(audio_headers.into_header_for_infinite_file()),
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
        headers: vec![
            ("Path".to_string(), "audio".to_string()),
            ("X-RequestId".to_string(), session_id.to_string()),
            ("X-Timestamp".to_string(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string()),
        ],
        data,
    }
}

#[cfg(test)]
mod tests {
    use crate::auth::Auth;
    use crate::recognizer::config::{Os, System, LanguageDetectMode, OutputFormat, Profanity, ResolverConfig};
    use crate::recognizer::utils::{generate_uri_for_stt_speech_azure};
    use serde_json::Value;
    use uuid::Uuid;
    use crate::recognizer::{AudioFormat, Details, Headers, Source};

    #[test]
    fn test_create_speech_config_message() {
        let (_, rx) = tokio::sync::mpsc::channel(1024);
        let source = Source::new(rx, Headers {
            sample_rate: 16000,
            bits_per_sample: 16,
            channels: 1,
            format: AudioFormat::PCM,
        }, Details::unknown());

        let mut config = ResolverConfig::new(Auth::from_subscription("westus", "my_subscription"));
        config.set_mode(crate::recognizer::config::RecognitionMode::Conversation);
        config.set_os(Os::unknown());
        config.set_system(System::unknown());

        let session_id = Uuid::new_v4();


        let message = crate::recognizer::utils::create_speech_config_message(session_id, &config, &source);
        match message {
            crate::connector::message::Message::Text { headers, data } => {
                assert_eq!(headers.len(), 4);
                assert_eq!(headers[0].0, "Path");
                assert_eq!(headers[0].1, "speech.config");
                assert_eq!(headers[1].0, "X-RequestId");
                assert_eq!(headers[1].1, session_id.to_string());
                assert_eq!(headers[2].0, "Content-Type");
                assert_eq!(headers[2].1, "application/json");
                assert_eq!(headers[3].0, "X-Timestamp");
                assert_eq!(data.is_some(), true);

                // test data
                assert_eq!(serde_json::from_str::<Value>(r#"{"context":{"audio":{"source":{"bitspersample":16,"channelcount":1,"connectivity":"Unknown","manufacturer":"Unknown","model":"Unknown","samplerate":16000,"type":"Unknown"}},"os":{"name":"Unknown","platform":"Unknown","version":"Unknown"},"system":{"build":"Unknown","lang":"Unknown","name":"Unknown","version":"Unknown"}},"recognition":"conversation"}"#).unwrap(), data.unwrap());
            }
            _ => panic!("Expected Text message")
        }
    }

    #[test]
    fn test_create_speech_context_message() {
        let mut config = ResolverConfig::new(Auth::from_subscription("westus", "my_subscription"));

        config.set_detect_languages(vec![String::from("en-us"), String::from("it-it")], LanguageDetectMode::Continuous);
        config.set_output_format(OutputFormat::Detailed);
        config.set_phrases(vec![String::from("hello world")]);
        config.set_custom_models(vec![("en-us".to_string(), "https://custom-model.com".to_string())]);

        let session_id = Uuid::new_v4();
        let message = crate::recognizer::utils::create_speech_context_message(session_id, &config);
        match message {
            crate::connector::message::Message::Text { headers, data } => {
                assert_eq!(headers.len(), 4);
                assert_eq!(headers[0].0, "Path");
                assert_eq!(headers[0].1, "speech.context");
                assert_eq!(headers[1].0, "X-RequestId");
                assert_eq!(headers[1].1, session_id.to_string());
                assert_eq!(headers[2].0, "Content-Type");
                assert_eq!(headers[2].1, "application/json");
                assert_eq!(headers[3].0, "X-Timestamp");
                assert_eq!(data.is_some(), true);

                // test data
                assert_eq!(serde_json::from_str::<Value>(r#"{"dgi":{"Groups":[{"Items":[{"Text":"hello world"}],"Type":"Generic"}]},"languageId":{"Priority":"PrioritizeLatency","languages":["en-us","it-it"],"mode":"DetectContinuous","onSuccess":{"action":"Recognize"},"onUnknown":{"action":"None"}},"phraseDetection":{"customModels":[{"endpoint":"https://custom-model.com","language":"en-us"}],"onInterim":null,"onSuccess":null},"phraseOutput":{"interimResults":{"resultType":"Auto"},"phraseResults":{"resultType":"Always"}}}"#).unwrap(), data.unwrap());
            }
            _ => panic!("Expected Text message")
        }
    }

    #[test]
    fn test_create_speech_audio_headers_message() {
        let session_id = Uuid::new_v4();

        let audio_headers = Headers {
            sample_rate: 16000,
            bits_per_sample: 16,
            channels: 1,
            format: AudioFormat::PCM,
        };

        let message = crate::recognizer::utils::create_speech_audio_headers_message(session_id, "audio/x-wav".to_string(), audio_headers.clone());
        match message {
            crate::connector::message::Message::Binary { headers, data } => {
                assert_eq!(headers.len(), 4);
                assert_eq!(headers[0].0, "Path");
                assert_eq!(headers[0].1, "audio");
                assert_eq!(headers[1].0, "X-RequestId");
                assert_eq!(headers[1].1, session_id.to_string());
                assert_eq!(headers[2].0, "Content-Type");
                assert_eq!(headers[2].1, "audio/x-wav");
                assert_eq!(headers[3].0, "X-Timestamp");
                assert_eq!(data.is_some(), true);

                // test data
                let audio_headers_vec: Vec<u8> = audio_headers.into();
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
            crate::connector::message::Message::Binary { headers, data } => {
                assert_eq!(headers.len(), 3);
                assert_eq!(headers[0].0, "Path");
                assert_eq!(headers[0].1, "audio");
                assert_eq!(headers[1].0, "X-RequestId");
                assert_eq!(headers[1].1, session_id.to_string());
                assert_eq!(headers[2].0, "X-Timestamp");
                assert_eq!(data.is_some(), true);

                // test data
                assert_eq!(data.unwrap(), audio);
            }
            _ => panic!("Expected Binary message")
        }

        let message = crate::recognizer::utils::create_speech_audio_message(session_id, None);
        match message {
            crate::connector::message::Message::Binary { headers, data } => {
                assert_eq!(headers.len(), 3);
                assert_eq!(headers[0].0, "Path");
                assert_eq!(headers[0].1, "audio");
                assert_eq!(headers[1].0, "X-RequestId");
                assert_eq!(headers[1].1, session_id.to_string());
                assert_eq!(headers[2].0, "X-Timestamp");
                assert_eq!(data.is_some(), false);
            }
            _ => panic!("Expected Binary message")
        }
    }

    #[test]
    fn generate_uri_for_stt_speech_azure_generates_correct_uri_for_us_region() {
        
        let mut config = ResolverConfig::new(Auth::from_subscription("westus", "my_subscription"));
        config.set_detect_languages(vec![String::from("en-us"), String::from("it-it")], LanguageDetectMode::Continuous);
        config.set_output_format(OutputFormat::Detailed);
        config.set_phrases(vec![String::from("hello world")]);
        config.set_profanity(Profanity::Masked);
        config.set_store_audio(true);

        let uri = generate_uri_for_stt_speech_azure(&config);

        let uri = url::Url::parse(uri.as_str()).unwrap();
        // tests path
        assert_eq!(uri.path(), "/speech/recognition/conversation/cognitiveservices/v1");
        // tests query parameters
        assert_eq!(uri.query_pairs().count(), 7);
        assert_eq!(uri.query_pairs().find(|x| x.0 == "Ocp-Apim-Subscription-Key").unwrap().1, "my_subscription");
        assert_eq!(uri.query_pairs().find(|x| x.0 == "language").unwrap().1, "en-us");
        assert_eq!(uri.query_pairs().find(|x| x.0 == "format").unwrap().1, "detailed");
        assert_eq!(uri.query_pairs().find(|x| x.0 == "profanity").unwrap().1, "masked");
        assert_eq!(uri.query_pairs().find(|x| x.0 == "storeAudio").unwrap().1, "true");
        assert_eq!(uri.query_pairs().find(|x| x.0 == "lidEnabled").unwrap().1, "true");
        assert_eq!(uri.query_pairs().find(|x| x.0 == "wordLevelTimestamps").unwrap().1, "true");
    }

    #[test]
    fn generate_uri_for_stt_speech_azure_generates_correct_uri_for_china_region() {
        
        let config = ResolverConfig::new(Auth::from_subscription("chinaeast", "my_subscription"));
        let uri = generate_uri_for_stt_speech_azure(&config);
        assert!(uri.starts_with("wss://chinaeast.stt.speech.azure.cn"));
    }

    #[test]
    fn generate_uri_for_stt_speech_azure_generates_correct_uri_for_usgov_region() {
        
        let config = ResolverConfig::new(Auth::from_subscription("usgovwest", "my_subscription"));

        let uri = generate_uri_for_stt_speech_azure(&config);
        assert!(uri.starts_with("wss://usgovwest.stt.speech.azure.us"));
    }

    #[test]
    fn generate_uri_for_stt_speech_azure_generates_correct_uri_for_multiple_languages() {
        
        let mut config = ResolverConfig::new(Auth::from_subscription("westus", "my_subscription"));
        config.set_detect_languages(vec![String::from("en-us"), String::from("es-es")], LanguageDetectMode::Continuous);
        let uri = generate_uri_for_stt_speech_azure(&config);
        assert!(uri.contains("lidEnabled=true"));
    }
}

