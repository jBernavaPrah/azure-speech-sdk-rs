use std::sync::Arc;
use crate::config::Device;
use crate::synthesizer::AudioFormat;
use crate::synthesizer::Language;
use crate::synthesizer::ssml::Voice;

#[derive(Clone, Default)]
pub struct Config
{
    pub(crate) output_format: AudioFormat,

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

    pub(crate) on_session_started: Option<Arc<Box<dyn Fn() + Send + Sync + 'static>>>,
    pub(crate) on_session_ended: Option<Arc<Box<dyn Fn() + Send + Sync + 'static>>>,
    pub(crate) on_synthesising: Option<Arc<Box<dyn Fn(Vec<u8>) + Send + Sync + 'static>>>,
    pub(crate) on_audio_metadata: Option<Arc<Box<dyn Fn(String) + Send + Sync + 'static>>>,
    pub(crate) on_synthesised: Option<Arc<Box<dyn Fn() + Send + Sync + 'static>>>,
    pub(crate) on_error: Option<Arc<Box<dyn Fn(crate::Error) + Send + Sync + 'static>>>,

}


impl Config
{
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

    pub fn with_output_format(mut self, output_format: AudioFormat) -> Self {
        self.output_format = output_format;
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

    pub fn on_session_start<Func>(mut self, func: Func) -> Self
    where
        Func: Send + Sync + 'static + Fn(),
    {
        self.on_session_started = Some(Arc::new(Box::new(func)));
        self
    }

    pub fn on_session_end<Func>(mut self, func: Func) -> Self
    where
        Func: Send + Sync + 'static + Fn(),
    {
        self.on_session_ended = Some(Arc::new(Box::new(func)));
        self
    }

    pub fn on_synthesising<Func>(mut self, func: Func) -> Self
    where
        Func: Send + Sync + 'static + Fn(Vec<u8>),
    {
        self.on_synthesising = Some(Arc::new(Box::new(func)));
        self
    }

    pub fn on_audio_metadata<Func>(mut self, func: Func) -> Self
    where
        Func: Send + Sync + 'static + Fn(String),
    {
        self.on_audio_metadata = Some(Arc::new(Box::new(func)));
        self
    }

    pub fn on_synthesised<Func>(mut self, func: Func) -> Self
    where
        Func: Send + Sync + 'static + Fn(),
    {
        self.on_synthesised = Some(Arc::new(Box::new(func)));
        self
    }

    pub fn on_error<Func>(mut self, func: Func) -> Self
    where
        Func: Send + Sync + 'static + Fn(crate::Error),
    {
        self.on_error = Some(Arc::new(Box::new(func)));
        self
    }
}
