use url::Url;
use crate::speech_to_text::config::RecognitionConfig;


pub(crate) fn get_azure_hostname_from_region(region: &String) -> String {
    if region.contains("china") { return String::from(".azure.cn"); }
    if region.to_lowercase().starts_with("usgov") { return String::from(".azure.us"); }

    return String::from(".microsoft.com");
}

pub(crate) fn generate_uri_for_stt_speech_azure(region: String, subscription: String, config: RecognitionConfig) -> String {

    let mut url = Url::parse(format!("wss://{}.stt.speech{}", region, get_azure_hostname_from_region(&region)).as_str()).unwrap();

    url.set_path(config.mode.to_uri_path().as_str());

    let lang = config.languages.first().expect("At least one language!");

    url.query_pairs_mut().append_pair("Ocp-Apim-Subscription-Key", subscription.to_string().as_str());
    url.query_pairs_mut().append_pair("language", lang.as_str());
    url.query_pairs_mut().append_pair("format", &config.output_format.as_str());
    url.query_pairs_mut().append_pair("profanity", &config.profanity.as_str());
    url.query_pairs_mut().append_pair("storeAudio", &config.store_audio.to_string());

    if let Some(advanced_config) = config.advanced_config {
        url.query_pairs_mut().append_pair("wordLevelTimestamps", advanced_config.word_level_timestamps.to_string().as_str());
    }

    if config.languages.len() > 1 {
        url.query_pairs_mut().append_pair("lidEnabled", true.to_string().as_str());
    }

    if config.connection_id.is_some() {
        url.query_pairs_mut().append_pair("X-ConnectionId", config.connection_id.unwrap().as_str());
    }

    url.to_string()
}

pub(crate) mod audio_header {
    use std::io::Cursor;
    use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};

    pub(crate) fn set_string(buffer: &mut [u8], offset: usize, value: &str) {
        let bytes = value.as_bytes();
        for (i, &byte) in bytes.iter().enumerate() {
            buffer[offset + i] = byte;
        }
    }

    pub(crate) fn set_u32(buffer: &mut [u8], offset: usize, value: u32) {
        let mut writer = Cursor::new(&mut buffer[offset..offset + 4]);
        writer.write_u32::<LittleEndian>(value).unwrap();
    }

    pub(crate) fn set_u16(buffer: &mut [u8], offset: usize, value: u16) {
        let mut writer = Cursor::new(&mut buffer[offset..offset + 2]);
        writer.write_u16::<LittleEndian>(value).unwrap();
    }

    pub(crate) fn get_u16(buffer: &[u8], offset: usize) -> u16 {
        let mut reader = Cursor::new(&buffer[offset..offset + 2]);
        reader.read_u16::<LittleEndian>().unwrap()
    }

    pub(crate) fn get_u32(buffer: &[u8], offset: usize) -> u32 {
        let mut reader = Cursor::new(&buffer[offset..offset + 4]);
        reader.read_u32::<LittleEndian>().unwrap()
    }
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
        let mut config = RecognitionConfig::new();
        config.languages = vec![String::from("en-us"), String::from("it-it")];
        config.output_format = OutputFormat::Detailed;
        config.profanity = Profanity::Masked;
        config.store_audio = false;
        let uri = generate_uri_for_stt_speech_azure(String::from("westus"), String::from("my_subscription"), config);

        let uri = url::Url::parse(uri.as_str()).unwrap();
        // test path
        assert_eq!(uri.path(), "/speech/recognition/conversation/cognitiveservices/v1");
        // test query parameters
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
        let mut config = RecognitionConfig::new();
        config.languages = vec![String::from("zh-cn")];
        let uri = generate_uri_for_stt_speech_azure(String::from("chinaeast"), String::from("my_subscription"), config);
        assert!(uri.starts_with("wss://chinaeast.stt.speech.azure.cn"));
    }

    #[test]
    fn generate_uri_for_stt_speech_azure_generates_correct_uri_for_usgov_region() {
        let mut config = RecognitionConfig::new();
        config.languages = vec![String::from("en-us")];
        let uri = generate_uri_for_stt_speech_azure(String::from("usgovwest"), String::from("my_subscription"), config);
        assert!(uri.starts_with("wss://usgovwest.stt.speech.azure.us"));
    }

    #[test]
    fn generate_uri_for_stt_speech_azure_generates_correct_uri_for_multiple_languages() {
        let mut config = RecognitionConfig::new();
        config.languages = vec![String::from("en-us"), String::from("es-es")];
        let uri = generate_uri_for_stt_speech_azure(String::from("westus"), String::from("my_subscription"), config);
        assert!(uri.contains("lidEnabled=true"));
    }

    #[test]
    fn generate_uri_for_stt_speech_azure_generates_correct_uri_for_advanced_config() {
        let mut config = RecognitionConfig::new();
        config.languages = vec![String::from("en-us")];
        config.advanced_config = Some(AdvancedConfig {
            word_level_timestamps: true,
        });
        let uri = generate_uri_for_stt_speech_azure(String::from("westus"), String::from("my_subscription"), config);
        assert!(uri.contains("wordLevelTimestamps=true"));
    }
}

