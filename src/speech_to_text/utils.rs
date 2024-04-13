use hound::WavSpec;
use url::Url;
use crate::speech_to_text::config::RecognitionConfig;

#[derive(Debug, Clone)]
pub enum AudioFormat {
    PCM,
    IEEE,
}

#[derive(Debug, Clone)]
pub struct AudioHeaders {
    pub bits_per_sample: u16,
    pub sample_rate: u32,
    pub channels: u16,
    pub format: AudioFormat,
}

impl AudioHeaders {

    pub fn new(format: AudioFormat, sample_rate: u32, bits_per_sample: u16, channels: u16) -> Self {
        Self {
            bits_per_sample,
            sample_rate,
            channels,
            format,
        }
    }
    pub fn to_vec(&self) -> Vec<u8> {
        self.to_hound_spec().into_header_for_infinite_file()
    }

    pub fn from_hound_spec(spec: WavSpec) -> Self {
        Self {
            bits_per_sample: spec.bits_per_sample,
            sample_rate: spec.sample_rate,
            channels: spec.channels,
            format: match spec.sample_format {
                hound::SampleFormat::Int => AudioFormat::PCM,
                hound::SampleFormat::Float => AudioFormat::IEEE,
            },
        }
    }

    pub fn to_hound_spec(&self) -> WavSpec {
        WavSpec {
            channels: self.channels,
            sample_rate: self.sample_rate,
            bits_per_sample: self.bits_per_sample,
            sample_format: match self.format {
                AudioFormat::PCM => hound::SampleFormat::Int,
                AudioFormat::IEEE => hound::SampleFormat::Float,
            },
        }
    }
}


pub(crate) fn get_azure_hostname_from_region(region: &String) -> String {
    if region.contains("china") { return String::from(".azure.cn"); }
    if region.to_lowercase().starts_with("usgov") { return String::from(".azure.us"); }

    return String::from(".microsoft.com");
}

pub(crate) fn generate_uri_for_stt_speech_azure(config: &RecognitionConfig) -> String {
    let mut url = Url::parse(format!("wss://{}.stt.speech{}", config.region, get_azure_hostname_from_region(&config.region)).as_str()).unwrap();

    url.set_path(config.mode.to_uri_path().as_str());

    let lang = config.languages.first().expect("At least one language!");

    url.query_pairs_mut().append_pair("Ocp-Apim-Subscription-Key", config.subscription.to_string().as_str());
    url.query_pairs_mut().append_pair("language", lang.as_str());
    url.query_pairs_mut().append_pair("format", &config.output_format.as_str());
    url.query_pairs_mut().append_pair("profanity", &config.profanity.as_str());
    url.query_pairs_mut().append_pair("storeAudio", &config.store_audio.to_string());

    if let Some(ref advanced_config) = config.advanced_config {
        url.query_pairs_mut().append_pair("wordLevelTimestamps", advanced_config.word_level_timestamps.to_string().as_str());
    }

    if config.languages.len() > 1 {
        url.query_pairs_mut().append_pair("lidEnabled", true.to_string().as_str());
    }

    if let Some(ref connection_id) = config.connection_id {
        url.query_pairs_mut().append_pair("X-ConnectionId", connection_id.as_str());
    }

    url.to_string()
}

#[cfg(test)]
mod tests {
    use crate::speech_to_text::config::{AdvancedConfig, OutputFormat, Profanity};
    use super::*;

    #[test]
    fn test_get_azure_hostname_from_region() {
        assert_eq!(get_azure_hostname_from_region(&String::from("fallback")), String::from(".microsoft.com"));
        assert_eq!(get_azure_hostname_from_region(&String::from("chinaeast")), String::from(".azure.cn"));
        assert_eq!(get_azure_hostname_from_region(&String::from("usgovwest")), String::from(".azure.us"));
    }

    #[test]
    fn generate_uri_for_stt_speech_azure_generates_correct_uri_for_us_region() {
        let mut config = RecognitionConfig::new("westus", "my_subscription");
        config.languages = vec![String::from("en-us"), String::from("it-it")];
        config.output_format = OutputFormat::Detailed;
        config.profanity = Profanity::Masked;
        config.store_audio = false;
        let uri = generate_uri_for_stt_speech_azure(&config);

        let uri = url::Url::parse(uri.as_str()).unwrap();
        // tests path
        assert_eq!(uri.path(), "/speech/recognition/conversation/cognitiveservices/v1");
        // tests query parameters
        assert_eq!(uri.query_pairs().count(), 6);
        assert_eq!(uri.query_pairs().find(|x| x.0 == "Ocp-Apim-Subscription-Key").unwrap().1, "my_subscription");
        assert_eq!(uri.query_pairs().find(|x| x.0 == "language").unwrap().1, "en-us");
        assert_eq!(uri.query_pairs().find(|x| x.0 == "format").unwrap().1, "detailed");
        assert_eq!(uri.query_pairs().find(|x| x.0 == "profanity").unwrap().1, "masked");
        assert_eq!(uri.query_pairs().find(|x| x.0 == "storeAudio").unwrap().1, "false");
        assert_eq!(uri.query_pairs().find(|x| x.0 == "lidEnabled").unwrap().1, "true");
    }

    #[test]
    fn generate_uri_for_stt_speech_azure_generates_correct_uri_for_china_region() {
        let mut config = RecognitionConfig::new("westus", "my_subscription");
        config.languages = vec![String::from("zh-cn")];
        let uri = generate_uri_for_stt_speech_azure(&config);
        assert!(uri.starts_with("wss://chinaeast.stt.speech.azure.cn"));
    }

    #[test]
    fn generate_uri_for_stt_speech_azure_generates_correct_uri_for_usgov_region() {
        let mut config = RecognitionConfig::new("westus", "my_subscription");
        config.languages = vec![String::from("en-us")];
        let uri = generate_uri_for_stt_speech_azure(&config);
        assert!(uri.starts_with("wss://usgovwest.stt.speech.azure.us"));
    }

    #[test]
    fn generate_uri_for_stt_speech_azure_generates_correct_uri_for_multiple_languages() {
        let mut config = RecognitionConfig::new("westus", "my_subscription");
        config.languages = vec![String::from("en-us"), String::from("es-es")];
        let uri = generate_uri_for_stt_speech_azure(&config);
        assert!(uri.contains("lidEnabled=true"));
    }

    #[test]
    fn generate_uri_for_stt_speech_azure_generates_correct_uri_for_advanced_config() {
        let mut config = RecognitionConfig::new("westus", "my_subscription");
        config.languages = vec![String::from("en-us")];
        config.advanced_config = Some(AdvancedConfig {
            word_level_timestamps: true,
        });
        let uri = generate_uri_for_stt_speech_azure(&config);
        assert!(uri.contains("wordLevelTimestamps=true"));
    }
}

