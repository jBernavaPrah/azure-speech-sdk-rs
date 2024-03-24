use serde::{Deserialize, Serialize};
use crate::speech_to_text::utils::audio_header::{get_u16, get_u32, set_string, set_u16, set_u32};

#[derive(Debug, Clone)]
pub(crate) struct RecognitionConfig {
    pub(crate) languages: Vec<String>,
    pub(crate) output_format: OutputFormat,
    pub(crate) mode: RecognitionMode,
    // when multiple languages are set, this value need to be different from None.
    pub(crate) language_detect_mode: Option<LanguageDetectMode>,
    /// store audio.
    pub(crate) store_audio: bool,
    pub(crate) profanity: Profanity,

    pub(crate) phrases: Option<Vec<String>>,

    pub(crate) connection_id: Option<String>,

    pub(crate) custom_models: Option<Vec<(String, String)>>,

    pub(crate) os: Os,
    pub(crate) system: System,
    pub(crate) source: Source,

    pub(crate) advanced_config: Option<AdvancedConfig>,
}

impl RecognitionConfig {
    pub(crate) fn new() -> Self {
        RecognitionConfig {
            mode: RecognitionMode::Conversation,
            languages: vec!["en-us".to_string()],
            output_format: OutputFormat::Simple,
            store_audio: false,
            phrases: None,
            profanity: Profanity::Masked,
            connection_id: None,

            language_detect_mode: None,
            custom_models: None,

            advanced_config: None,

            source: Source::default(),

            system: System::default(),
            os: Os::default(),
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct System {
    pub name: String,
    pub version: String,
    pub build: String,
    pub lang: String,
}

impl System {
    pub fn default() -> Self {
        System {
            name: "rust".to_string(),
            build: "rust".to_string(),
            version: "0.0.1".to_string(),
            lang: "Rust".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Os {
    pub platform: String,
    pub name: String,
    pub version: String,
}

impl Os {
    pub fn default() -> Self {
        let os = os_info::get();
        Os {
            version: os.version().to_string(),
            name: os.os_type().to_string(),
            platform: os.to_string(),
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub name: String,
    pub model: String,
    pub connectivity: String,
    pub manufacturer: String,
}

impl Source {
    pub fn default() -> Self {
        Source {
            name: "Unknown".to_string(),
            model: "Unknown".to_string(),
            manufacturer: "Unknown".to_string(),
            connectivity: "Unknown".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Silence {
    pub initial_timeout_ms: Option<i32>,
    pub end_timeout_ms: Option<i32>,
    pub segmentation_timeout_ms: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct AdvancedConfig {
    pub word_level_timestamps: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AudioFormat {
    PCM,
    MuLaw,
    Siren,
    MP3,
    SILKSkype,
    OggOpus,
    WebmOpus,
    ALaw,
    FLAC,
    OPUS,
    None,
}

impl From<u16> for AudioFormat {
    fn from(value: u16) -> Self {
        match value {
            1 => AudioFormat::PCM,
            6 => AudioFormat::ALaw,
            7 => AudioFormat::MuLaw,
            _ => AudioFormat::None
        }
    }
}

impl Into<u16> for AudioFormat {
    fn into(self) -> u16 {
        match self {
            AudioFormat::PCM => 1,
            AudioFormat::ALaw => 6,
            AudioFormat::MuLaw => 7,
            _ => 0
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioHeaders {
    #[serde(skip_serializing)]
    pub format: AudioFormat,

    #[serde(rename = "samplerate")]
    pub sample_rate: u32,
    #[serde(rename = "bitspersample")]
    pub bits_per_sample: u16,
    #[serde(rename = "channelcount")]
    pub channel_count: u16,

}

impl From<Vec<u8>> for AudioHeaders {
    fn from(value: Vec<u8>) -> Self {
        AudioHeaders {
            format: get_u16(&value, 20).into(),
            channel_count: get_u16(&value, 22),
            sample_rate: get_u32(&value, 24),
            bits_per_sample: get_u16(&value, 34),
        }
    }
}

impl Into<Vec<u8>> for AudioHeaders {
    fn into(self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![0; 44];

        // RIFF identifier
        set_string(&mut buffer, 0, "RIFF");

        // file length
        set_u32(&mut buffer, 4, 0);

        // RIFF type & Format
        set_string(&mut buffer, 8, "WAVEfmt ");

        // format chunk length
        set_u32(&mut buffer, 16, 16);

        // audio format
        set_u16(&mut buffer, 20, 1); // Assuming PCM format

        // channel count
        set_u16(&mut buffer, 22, 1); // Assuming mono channel

        // sample rate
        set_u32(&mut buffer, 24, 16000); // Assuming 16KHz

        // byte rate (sample rate * block align)
        set_u32(&mut buffer, 28, 16000 * 1 * 2); // Assuming 16KHz, mono channel, 16 bits per sample

        // block align (channel count * bytes per sample)
        set_u16(&mut buffer, 32, 1 * 2); // Assuming mono channel, 16 bits per sample

        // bits per sample
        set_u16(&mut buffer, 34, 16); // Assuming 16 bits per sample

        // data chunk identifier
        set_string(&mut buffer, 36, "data");

        // data chunk length
        set_u32(&mut buffer, 40, 0);

        buffer
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RecognitionMode {
    #[serde(rename = "conversation")]
    Conversation,
    /// Do not use this mode. It is not supported yet
    #[serde(rename = "interactive")]
    Interactive,
    /// Do not use this mode. It is not supported yet
    #[serde(rename = "dictation")]
    Dictation,
}


impl RecognitionMode {
    pub(crate) fn to_uri_path(self) -> String {
        match self {
            RecognitionMode::Conversation => String::from("/speech/recognition/conversation/cognitiveservices/v1"),
            RecognitionMode::Interactive => String::from("/speech/recognition/interactive/cognitiveservices/v1"),
            RecognitionMode::Dictation => String::from("/speech/recognition/dictation/cognitiveservices/v1"),
        }
    }
}


#[derive(Debug, Clone)]
pub enum OutputFormat {
    Simple,
    Detailed,
}

impl OutputFormat {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            OutputFormat::Simple => "simple",
            OutputFormat::Detailed => "detailed"
        }
    }
}

#[derive(Debug, Clone)]
pub enum Profanity {
    Masked,
    Removed,
    Raw,
}


impl Profanity {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Profanity::Masked => "masked",
            Profanity::Removed => "removed",
            Profanity::Raw => "raw"
        }
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LanguageDetectMode {
    #[serde(rename = "DetectContinuous")]
    Continuous,
    #[serde(rename = "DetectAtAudioStart")]
    AtStart,
}
