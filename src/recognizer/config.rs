use crate::config::Device;
use crate::recognizer::Language;
use serde::{Deserialize, Serialize};

/// The configuration for the recognizer.
///
/// The configuration is used to set the parameters of the speech recognition.
#[derive(Clone, Debug)]
pub struct Config {
    pub(crate) device: Device,

    pub(crate) languages: Vec<Language>,
    pub(crate) output_format: OutputFormat,

    // todo: probably this will be removed and moved directly in the connection.
    pub(crate) mode: RecognitionMode, // todo: what is this?

    pub(crate) language_detect_mode: Option<LanguageDetectMode>,

    pub(crate) phrases: Option<Vec<String>>,

    pub(crate) custom_models: Option<Vec<(String, String)>>,

    pub(crate) connection_id: Option<String>, // todo: what is this for?

    pub(crate) store_audio: bool, // todo: is this needed?

    pub(crate) profanity: Profanity,
    // todo: check diarization https://learn.microsoft.com/en-us/azure/ai-services/speech-service/get-started-stt-diarization?tabs=macos&pivots=programming-language-javascript
    // probably will be moved from here and added to a separate module.
    //pub(crate) recognize_speaker: bool,

    // todo add more detailed configuration from default:  src/common.speech/ConnectionFactoryBase.ts
}

impl Default for Config {
    fn default() -> Self {
        Config {
            languages: vec![Language::default()],
            output_format: OutputFormat::Simple,
            mode: RecognitionMode::Conversation,
            language_detect_mode: None,
            phrases: None,
            custom_models: None,
            connection_id: None,
            store_audio: false,
            device: Device::default(),
            profanity: Profanity::Masked,
        }
    }
}

impl Config {
    /// Enable audio logging in service.
    ///
    /// Audio and content logs are stored either in Microsoft-owned storage, or in your own storage account linked
    /// to your Cognitive Services subscription (Bring Your Own Storage (BYOS) enabled Speech resource).
    /// The logs will be removed after 30 days.
    pub fn enable_audio_logging(mut self) -> Self {
        self.store_audio = true;
        self
    }

    /// Set Device information.
    ///
    /// The device information is used to provide information about the source.
    /// Some default values are already set.
    pub fn set_device(mut self, device: Device) -> Self {
        self.device = device;
        self
    }

    /// Mask the profanity.
    pub fn set_profanity(mut self, profanity: Profanity) -> Self {
        self.profanity = profanity;
        self
    }

    /// Set the default language for the recognition.
    ///
    /// If needed multiple language detection, use the set_detect_languages method.
    pub fn set_language(mut self, language: Language) -> Self {
        self.languages = vec![language];
        self
    }

    /// Instruct to detect the languages from the audio.
    ///
    /// The language detection is used to detect the language of the audio.
    /// This could not match the language of the audio, but it is used to provide better recognition.
    pub fn set_detect_languages(
        mut self,
        languages: Vec<Language>,
        language_detect_mode: LanguageDetectMode,
    ) -> Self {
        self.languages = languages;
        self.language_detect_mode = Some(language_detect_mode);
        self
    }

    /// Helping phrases to detect better the context.
    ///
    /// Untested.
    pub fn set_phrases(mut self, phrases: Vec<String>) -> Self {
        self.phrases = Some(phrases);
        self
    }

    /// Use custom Models.
    ///
    /// Untested.
    pub fn set_custom_models(mut self, custom_models: Vec<(String, String)>) -> Self {
        self.custom_models = Some(custom_models);
        self
    }

    /// Set the recognition mode.
    ///
    /// *Only the Conversation mode was tested.*
    #[allow(dead_code)]
    pub fn set_recognition_mode(mut self, mode: RecognitionMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set the output format of event responses.
    ///
    /// You will find the json in each event with Message.json() method.
    pub fn set_output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }

    //
    // pub fn enable_recognize_speaker(mut self) -> Self {
    //     self.recognize_speaker = true;
    //     self
    // }
}

#[derive(Debug, Clone, Default)]
/// The profanity level.
pub enum Profanity {
    #[allow(missing_docs)]
    #[default]
    Masked,
    #[allow(missing_docs)]
    Removed,
    #[allow(missing_docs)]
    Raw,
}

impl Profanity {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Profanity::Masked => "masked",
            Profanity::Removed => "removed",
            Profanity::Raw => "raw",
        }
    }
}

#[derive(Debug, Clone)]
/// The configuration for the silence detection.
///
/// Untested.
pub struct Silence {
    #[allow(missing_docs)]
    pub initial_timeout_ms: Option<i32>,
    #[allow(missing_docs)]
    pub end_timeout_ms: Option<i32>,
    #[allow(missing_docs)]
    pub segmentation_timeout_ms: Option<i32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
/// The recognition mode.
pub enum RecognitionMode {
    /// Use this mode for normal conversation.
    #[serde(rename = "conversation")]
    #[default]
    Conversation,
    /// Untested.
    #[serde(rename = "interactive")]
    Interactive,
    /// Untested.
    #[serde(rename = "dictation")]
    Dictation,
}

impl RecognitionMode {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            RecognitionMode::Conversation => "conversation",
            RecognitionMode::Interactive => "interactive",
            RecognitionMode::Dictation => "dictation",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
/// The output format of the messages.
pub enum OutputFormat {
    #[allow(missing_docs)]
    #[default]
    Simple,
    #[allow(missing_docs)]
    Detailed,
}

impl OutputFormat {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            OutputFormat::Simple => "simple",
            OutputFormat::Detailed => "detailed",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
/// The primary language of the recognized text.
pub enum LanguageDetectMode {
    /// Detect the language at the start of the audio.
    #[serde(rename = "DetectContinuous")]
    #[default]
    Continuous,
    /// Detect the language at the start of the audio.
    #[serde(rename = "DetectAtAudioStart")]
    AtStart,
}

#[derive(Debug, Clone)]
/// Details of the source.
///
/// This is used to provide information about the source.
pub struct Details {
    /// Name of the source, e.g. "Microphone", "Stream", "File"
    pub name: String,
    /// Model of the source, e.g. "Stream", "File"
    pub model: String,
    /// Manufacturer of the source, e.g. "Unknown"
    pub connectivity: String,
    /// Connectivity of the source, e.g. "Unknown"
    pub manufacturer: String,
}

impl Details {
    /// Create a new Details instance
    pub fn new(
        name: impl Into<String>,
        model: impl Into<String>,
        manufacturer: impl Into<String>,
        connectivity: impl Into<String>,
    ) -> Self {
        Details {
            name: name.into(),
            model: model.into(),
            manufacturer: manufacturer.into(),
            connectivity: connectivity.into(),
        }
    }

    #[allow(missing_docs)]
    pub fn unknown() -> Self {
        Details::new("Unknown", "Unknown", "Unknown", "Unknown")
    }

    #[allow(missing_docs)]
    pub fn stream(manufacture: impl Into<String>, connectivity: impl Into<String>) -> Self {
        Details::new("Stream", "Stream", manufacture, connectivity)
    }
    #[allow(missing_docs)]
    pub fn microphone(manufacture: impl Into<String>, connectivity: impl Into<String>) -> Self {
        Details::new("Microphone", "Stream", manufacture, connectivity)
    }
    #[allow(missing_docs)]
    pub fn file() -> Self {
        Details::new("File", "File", "Unknown", "Unknown")
    }
}
