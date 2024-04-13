use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub(crate) struct RecognitionConfig {
    pub(crate) region: String,
    pub(crate) subscription: String,

    pub(crate) languages: Vec<String>,
    pub(crate) output_format: OutputFormat,

    pub(crate) mode: RecognitionMode,
    // todo: when multiple languages are set, this value need to be different from None.
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
    pub(crate) fn new(region: impl Into<String>, subscription: impl Into<String>) -> Self {
        RecognitionConfig {
            region: region.into(),
            subscription: subscription.into(),
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

            source: Source::unknown(),

            system: System::default(),
            os: Os::current(),
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
            name: env!("CARGO_PKG_NAME").to_string(),
            build: "rust".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            lang: "rust".to_string(),
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
    pub fn current() -> Self {
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
    pub fn unknown() -> Self {
        Source {
            name: "Unknown".to_string(),
            model: "Unknown".to_string(),
            manufacturer: "Unknown".to_string(),
            connectivity: "Unknown".to_string(),
        }
    }

    pub fn file() -> Self {
        Source {
            name: "File".to_string(),
            model: "File".to_string(),
            manufacturer: "Unknown".to_string(),
            connectivity: "Unknown".to_string(),
        }
    }
    pub fn microphone() -> Self {
        Source {
            name: "Stream".to_string(),
            model: "Microphone".to_string(),
            manufacturer: "Unknown".to_string(),
            connectivity: "Unknown".to_string(),
        }
    }

    pub fn stream() -> Self {
        Source {
            name: "Stream".to_string(),
            model: "File".to_string(),
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
