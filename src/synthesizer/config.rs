use crate::config::Device;
use crate::synthesizer::{AudioFormat, Language, Voice};

#[derive(Clone, Default, Debug)]
pub struct Config {
    pub(crate) audio_format: AudioFormat,

    pub(crate) device: Device,

    pub(crate) language: Language,
    pub(crate) voice: Option<Voice>,

    pub(crate) bookmark_enabled: bool,
    pub(crate) word_boundary_enabled: bool,
    pub(crate) punctuation_boundary_enabled: bool,
    pub(crate) sentence_boundary_enabled: bool,
    pub(crate) session_end_enabled: bool,
    pub(crate) viseme_enabled: bool,

    pub(crate) auto_detect_language: bool,
}

impl Config {
    pub fn new() -> Self {
        Self {
            session_end_enabled: true,
            auto_detect_language: true,
            ..Default::default()
        }
    }

    pub fn with_language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    pub fn with_voice(mut self, voice: Voice) -> Self {
        self.voice = Some(voice);
        self
    }

    pub fn with_audio_format(mut self, output_format: AudioFormat) -> Self {
        self.audio_format = output_format;
        self
    }

    pub fn enable_bookmark(mut self) -> Self {
        self.bookmark_enabled = true;
        self
    }

    pub fn enable_word_boundary(mut self) -> Self {
        self.word_boundary_enabled = true;
        self
    }

    pub fn enable_punctuation_boundary(mut self) -> Self {
        self.punctuation_boundary_enabled = true;
        self
    }

    pub fn enable_sentence_boundary(mut self) -> Self {
        self.sentence_boundary_enabled = true;
        self
    }

    pub fn enable_session_end(mut self) -> Self {
        self.session_end_enabled = true;
        self
    }

    pub fn enable_viseme(mut self) -> Self {
        self.viseme_enabled = true;
        self
    }

    pub fn disable_auto_detect_language(mut self) -> Self {
        self.auto_detect_language = false;
        self
    }

    pub fn set_device(mut self, device: Device) -> Self {
        self.device = device;
        self
    }
}
