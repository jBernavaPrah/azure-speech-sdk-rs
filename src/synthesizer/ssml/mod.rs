mod speak;

use std::fmt::Debug;

pub use ssml::*;

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
            [ssml::voice(voice.as_str(), [self])],
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

impl ToSSML for speak::Speak {
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

impl ToSSML for Speak {
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
