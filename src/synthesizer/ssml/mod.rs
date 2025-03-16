use std::fmt::Debug;

pub use ssml;
use ssml::{Serialize, SerializeOptions};

/// Trait to convert a type to SSML
///
/// It's already implemented for some basic types.
pub trait ToSSML: Debug {
    /// Convert the type to SSML
    ///
    /// The function will be called with the language and voice from the config.
    /// Up to you to use them or not.
    fn to_ssml(
        &self,
        language: crate::synthesizer::Language,
        voice: crate::synthesizer::Voice,
    ) -> crate::Result<String>;
}

impl ToSSML for String {
    fn to_ssml(
        &self,
        language: crate::synthesizer::Language,
        voice: crate::synthesizer::Voice,
    ) -> crate::Result<String> {
        serialize_to_ssml(&ssml::speak(
            Some(language.as_str()),
            [ssml::voice(voice.as_str(), [self.clone()])],
        ))
    }
}

impl ToSSML for &String {
    fn to_ssml(
        &self,
        language: crate::synthesizer::Language,
        voice: crate::synthesizer::Voice,
    ) -> crate::Result<String> {
        serialize_to_ssml(&ssml::speak(
            Some(language.as_str()),
            [ssml::voice(voice.as_str(), [self.as_str()])],
        ))
    }
}

impl ToSSML for str {
    fn to_ssml(
        &self,
        language: crate::synthesizer::Language,
        voice: crate::synthesizer::Voice,
    ) -> crate::Result<String> {
        serialize_to_ssml(&ssml::speak(
            Some(language.as_str()),
            [ssml::voice(voice.as_str(), [self.to_string()])],
        ))
    }
}

impl ToSSML for &str {
    fn to_ssml(
        &self,
        language: crate::synthesizer::Language,
        voice: crate::synthesizer::Voice,
    ) -> crate::Result<String> {
        serialize_to_ssml(&ssml::speak(
            Some(language.as_str()),
            [ssml::voice(voice.as_str(), [self.to_string()])],
        ))
    }
}

impl ToSSML for Speak {
    fn to_ssml(
        &self,
        language: crate::synthesizer::Language,
        voice: crate::synthesizer::Voice,
    ) -> crate::Result<String> {
        let language = self.language.as_ref().unwrap_or(&language);
        let voice = self.voice.as_ref().unwrap_or(&voice);

        serialize_to_ssml(&ssml::speak(
            Some(language.as_str()),
            [ssml::voice(voice.as_str(), [self.text.clone()])],
        ))
    }
}

impl ToSSML for ssml::Speak<'_> {
    fn to_ssml(
        &self,
        _language: crate::synthesizer::Language,
        _voice: crate::synthesizer::Voice,
    ) -> crate::Result<String> {
        serialize_to_ssml(self)
    }
}

fn serialize_to_ssml(speak: &impl Serialize) -> crate::Result<String> {
    speak
        .serialize_to_string(
            &SerializeOptions::default()
                .flavor(ssml::Flavor::MicrosoftAzureCognitiveSpeechServices),
        )
        .map_err(|e| crate::Error::InternalError(e.to_string()))
}

// Easy to use struct to create SSML speak elements.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Speak {
    pub(crate) text: String,
    pub(crate) voice: Option<crate::synthesizer::Voice>,
    pub(crate) language: Option<crate::synthesizer::Language>,
}

impl Speak {
    #[allow(dead_code)]
    pub fn new(text: String) -> Self {
        Self {
            text,
            voice: None,
            language: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_voice(mut self, voice: crate::synthesizer::Voice) -> Self {
        self.voice = Some(voice);
        self
    }

    #[allow(dead_code)]
    pub fn with_language(mut self, language: crate::synthesizer::Language) -> Self {
        self.language = Some(language);
        self
    }
}
