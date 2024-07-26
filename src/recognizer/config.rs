
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
// todo: use the state pattern to manage languages and language_detect_mode.
pub struct RecognizerConfig {

    pub(crate) languages: Vec<String>,
    pub(crate) output_format: OutputFormat,

    pub(crate) mode: RecognitionMode, // todo: what is this?

    pub(crate) language_detect_mode: Option<LanguageDetectMode>, // todo: when multiple languages are set, this value need to be different from None.

    pub(crate) phrases: Option<Vec<String>>,

    pub(crate) custom_models: Option<Vec<(String, String)>>,

    pub(crate) connection_id: Option<String>, // what is this for?
    
    pub(crate) store_audio: bool, // todo: is this needed?

    pub(crate) profanity: Profanity,
    
    //pub(crate) recognize_speaker: bool,
    
    // todo add more detailed configuration from default:  src/common.speech/ConnectionFactoryBase.ts
    
}

impl RecognizerConfig {
    /// Enable audio logging in service.
    /// Audio and content logs are stored either in Microsoft-owned storage, or in your own storage account linked
    /// to your Cognitive Services subscription (Bring Your Own Storage (BYOS) enabled Speech resource).
    /// The logs will be removed after 30 days.
    pub fn enable_audio_logging(mut self) -> Self {
        self.store_audio = true;
        self
    }

    /// Mask the profanity.
    pub fn set_profanity(mut self, profanity: Profanity) -> Self {
        self.profanity = profanity;
        self
    }

    /// Set the default language for the recognition.
    /// If needed multiple language detection, use the set_detect_languages method.
    pub fn set_language(mut self, language: impl Into<String>) -> Self {
        self.languages = vec![language.into()];
        self
    }

    /// Instruct to detect the languages from the audio.
    pub fn set_detect_languages(mut self,
                                languages: Vec<impl Into<String>>,
                                language_detect_mode: LanguageDetectMode,
    ) -> Self {
        self.languages = languages.into_iter().map(|l| l.into()).collect();
        self.language_detect_mode = Some(language_detect_mode);
        self
    }

    /// Helping phrases to detect better the context.
    /// Untested.
    pub fn set_phrases(mut self, phrases: Vec<String>) -> Self {
        self.phrases = Some(phrases);
        self
    }


    /// Use custom Models.
    /// Untested.
    pub fn set_custom_models(mut self, custom_models: Vec<(String, String)>) -> Self {
        self.custom_models = Some(custom_models);
        self
    }

    /// Set the recognition mode.
    /// Currently only the Conversation mode was tested.
    #[allow(dead_code)]
    pub(crate) fn set_mode(mut self, mode: RecognitionMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set the output format of event responses.
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

impl Default for RecognizerConfig {
    fn default() -> Self {
        RecognizerConfig {
            store_audio: false,
            profanity: Profanity::Masked,
            languages: vec!["en-us".to_string()],
            output_format: OutputFormat::Simple,
            mode: RecognitionMode::Conversation,
            language_detect_mode: None,
            phrases: None,
            custom_models: None,
            connection_id: None,
            // recognize_speaker: false,
        }
    }
}


#[derive(Debug, Clone)]
/// The profanity level.
pub enum Profanity {
    #[allow(missing_docs)]
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
            Profanity::Raw => "raw"
        }
    }
}



#[derive(Debug, Clone)]
/// The configuration for the silence detection.
/// Untested.
pub struct Silence {
    #[allow(missing_docs)]
    pub initial_timeout_ms: Option<i32>,
    #[allow(missing_docs)]
    pub end_timeout_ms: Option<i32>,
    #[allow(missing_docs)]
    pub segmentation_timeout_ms: Option<i32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
/// The recognition mode.
pub enum RecognitionMode {
    /// Use this mode for normal conversation.
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
    pub(crate) fn to_uri_path(self) -> &'static str {
        match self {
            RecognitionMode::Conversation => "/speech/recognition/conversation/cognitiveservices/v1",
            RecognitionMode::Interactive => "/speech/recognition/interactive/cognitiveservices/v1",
            RecognitionMode::Dictation => "/speech/recognition/dictation/cognitiveservices/v1",
        }
    }
}


#[derive(Debug, Clone, Eq, PartialEq)]
/// The output format of the events.
/// After you set the outputFormat, Service will return in the raw Message.json() the Sample or Detailed version of the json.
pub enum OutputFormat {
    #[allow(missing_docs)]
    Simple,
    #[allow(missing_docs)]
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


#[derive(Serialize, Deserialize, Clone, Debug)]
/// The primary language of the recognized text.
pub enum LanguageDetectMode {
    /// Detect the language at the start of the audio.
    #[serde(rename = "DetectContinuous")]
    Continuous,
    /// Detect the language at the start of the audio.
    #[serde(rename = "DetectAtAudioStart")]
    AtStart,
}

