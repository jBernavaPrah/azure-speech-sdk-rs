use crate::synthesizer::ssml::{Language, Voice};

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
    pub fn with_voice(mut self, voice: Voice) -> Self {
        self.voice = Some(voice);
        self
    }
    pub fn with_language(mut self, language: Language) -> Self {
        self.language = Some(language);
        self
    }
}
