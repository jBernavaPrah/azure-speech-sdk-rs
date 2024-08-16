use crate::synthesizer::{Language, Voice};

// Easy to use struct to create SSML speak elements.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Speak {
    pub(crate) text: String,
    pub(crate) voice: Option<Voice>,
    pub(crate) language: Option<Language>,
}

impl Speak {
    pub fn new(text: String) -> Self {
        Self {
            text,
            voice: None,
            language: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_voice(mut self, voice: Voice) -> Self {
        self.voice = Some(voice);
        self
    }

    #[allow(dead_code)]
    pub fn with_language(mut self, language: Language) -> Self {
        self.language = Some(language);
        self
    }
}
