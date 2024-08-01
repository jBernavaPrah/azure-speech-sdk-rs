use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use crate::config::Device;
use crate::Error;
use crate::synthesizer::AudioFormat;
use crate::synthesizer::Language;
use crate::synthesizer::ssml::Voice;

//

#[async_trait::async_trait]
pub trait Callback
{
    async fn on_session_start(&self) -> () {}
    async fn on_audio_chunk(&self, _audio_chunk: Vec<u8>) -> () {}
    async fn on_audio_metadata(&self, _audio_metadata: String) -> () {}
    async fn on_error(&self, _error: Error) -> () {}
    async fn on_session_end(&self) -> () {}
}


#[derive(Clone, Default)]
pub struct Config
{
    pub(crate) output_format: AudioFormat,

    pub(crate) device: Device,

    pub(crate) voice: Voice,
    pub(crate) language: Language,

    pub(crate) bookmark_enabled: bool,
    pub(crate) word_boundary_enabled: bool,
    pub(crate) punctuation_boundary_enabled: bool,
    pub(crate) sentence_boundary_enabled: bool,
    pub(crate) session_end_enabled: bool,
    pub(crate) viseme_enabled: bool,

    pub(crate) auto_detect_language: bool,

    //pub(crate) callbacks: Arc<Box<dyn Callback + Send>>,
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
        self.voice = voice;
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
    // 
    // pub fn on_session_start<Fut, F: Fn() -> Fut + Send + 'static>(mut self, f: F) -> Self
    // where
    //     Fut: Future<Output=()> + Send + 'static,
    // {
    //     self.on_session_start = Some(Arc::new(Box::new(move || Box::pin(f()))));
    //     self
    // }
    // 
    // pub fn on_audio_chunk<F: Fn(Vec<u8>) -> Pin<Box<dyn Future<Output=()> + Send>> + Send + 'static>(mut self, f: F) -> Self
    // {
    //     self.on_audio_chunk = Some(Arc::new(Box::new(move |v| Box::pin(f(v)))));
    //     self
    // }
    // 
    // pub fn on_audio_metadata<F: Fn(String) -> Pin<Box<dyn Future<Output=()> + Send>> + Send + 'static>(mut self, f: F) -> Self
    // {
    //     self.on_audio_metadata = Some(Arc::new(Box::new(f)));
    //     self
    // }
    // 
    // pub fn on_error<F: Fn(Error) -> Pin<Box<dyn Future<Output=()> + Send>> + Send + 'static>(mut self, f: F) -> Self
    // {
    //     self.on_error = Some(Arc::new(Box::new(f)));
    //     self
    // }
    // 
    // pub fn on_session_end<F: Fn() -> Pin<Box<dyn Future<Output=()> + Send>> + Send + 'static>(mut self, f: F) -> Self
    // {
    //     self.on_session_end = Some(Arc::new(Box::new(f)));
    //     self
    // }
}
