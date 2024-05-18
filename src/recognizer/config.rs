use serde::{Deserialize, Serialize};
use crate::auth::Auth;

#[derive(Debug)]
/// The configuration for the recognition.
pub struct ResolverConfig {
    pub(crate) auth: Auth,

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
}

impl ResolverConfig {
    
    /// Create a new configuration with the given auth.
    pub fn new(auth: Auth) -> Self {
        ResolverConfig {
            auth,
            mode: RecognitionMode::Conversation,
            languages: vec!["en-us".to_string()],
            output_format: OutputFormat::Simple,
            store_audio: false,
            phrases: None,
            profanity: Profanity::Masked,
            connection_id: None,

            language_detect_mode: None,
            custom_models: None,

            system: System::default(),
            os: Os::current(),
        }
    }

    /// Set the default language for the recognition.
    /// If needed multiple language detection, use the set_detect_languages method. 
    pub fn set_language(&mut self, language: impl Into<String>) -> &mut Self {
        self.languages = vec![language.into()];
        self
    }

    /// Instruct to detect the languages from the audio.
    pub fn set_detect_languages(&mut self,
                                languages: Vec<impl Into<String>>,
                                language_detect_mode: LanguageDetectMode,
    ) -> &mut Self {
        self.languages = languages.into_iter().map(|l| l.into()).collect();
        self.language_detect_mode = Some(language_detect_mode);
        self
    }
    
    /// Helping phrases to detect better the context.
    /// Untested.
    pub fn set_phrases(&mut self, phrases: Vec<String>) -> &mut Self {
        self.phrases = Some(phrases);
        self
    }
    
    /// Store the audio.
    /// Untested.
    pub fn set_store_audio(&mut self, store: bool) -> &mut Self {
        self.store_audio = store;
        self
    }

    /// Mask the profanity.
    pub fn set_profanity(&mut self, profanity: Profanity) -> &mut Self {
        self.profanity = profanity;
        self
    }
    
    /// Overwrite the OS information.
    /// This information is taken automatically from the system. But you can overwrite it.
    pub fn set_os(&mut self, os: Os) -> &mut Self {
        self.os = os;
        self
    }

    /// Overwrite the System information.
    /// This information is taken automatically from the system. But you can overwrite it.
    pub fn set_system(&mut self, system: System) -> &mut Self {
        self.system = system;
        self
    }

    /// Use custom Models. 
    /// Untested.
    pub fn set_custom_models(&mut self, custom_models: Vec<(String, String)>) -> &mut Self {
        self.custom_models = Some(custom_models);
        self
    }

    /// Set the recognition mode. 
    /// Currently only the Conversation mode was tested. 
    pub(crate) fn set_mode(&mut self, mode: RecognitionMode) -> &mut Self {
        self.mode = mode;
        self
    }
    
    /// Set the output format of event responses. 
    /// You will find the json in each event with Message.json() method. 
    pub fn set_output_format(&mut self, format: OutputFormat) -> &mut Self {
        self.output_format = format;
        self
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

    pub fn unknown() -> Self {
        System {
            name: "Unknown".to_string(),
            build: "Unknown".to_string(),
            version: "Unknown".to_string(),
            lang: "Unknown".to_string(),
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

    pub fn unknown() -> Self {
        Os {
            version: "Unknown".to_string(),
            name: "Unknown".to_string(),
            platform: "Unknown".to_string(),
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


#[derive(Debug, Clone, Eq, PartialEq)]
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
