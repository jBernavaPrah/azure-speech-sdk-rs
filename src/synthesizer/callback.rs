use crate::callback::{BoxFuture, OnError, OnSessionEnded, OnSessionStarted};
use crate::synthesizer::{message, Event};
use crate::RequestId;
use std::future::Future;
use std::sync::Arc;

pub(crate) type OnSynthesising = Arc<Box<dyn Fn(RequestId, Vec<u8>) -> BoxFuture>>;
pub(crate) type OnAudioMetadata = Arc<Box<dyn Fn(RequestId, Vec<message::Metadata>) -> BoxFuture>>;
pub(crate) type OnSynthesised = Arc<Box<dyn Fn(RequestId) -> BoxFuture>>;

#[derive(Default, Clone)]

pub struct Callback {
    pub(crate) on_session_started: Option<Arc<OnSessionStarted>>,
    pub(crate) on_error: Option<Arc<OnError>>,
    pub(crate) on_session_ended: Option<Arc<OnSessionEnded>>,

    pub(crate) on_synthesising: Option<OnSynthesising>,
    pub(crate) on_audio_metadata: Option<OnAudioMetadata>,
    pub(crate) on_synthesised: Option<OnSynthesised>,
}

impl Callback {
    pub fn on_session_start<F, Fut>(mut self, func: F) -> Self
    where
        F: Fn(RequestId) -> Fut + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        self.on_session_started = Some(Arc::new(Box::new(move |str| Box::pin(func(str)))));
        self
    }

    pub fn on_session_end<F, Fut>(mut self, func: F) -> Self
    where
        F: Fn(RequestId) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.on_session_ended = Some(Arc::new(Box::new(move |request_id| {
            Box::pin(func(request_id))
        })));
        self
    }

    pub fn on_error<F, Fut>(mut self, func: F) -> Self
    where
        F: Fn(RequestId, crate::Error) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.on_error = Some(Arc::new(Box::new(move |request, err| {
            Box::pin(func(request, err))
        })));
        self
    }

    pub fn on_synthesising<F, Fut>(mut self, func: F) -> Self
    where
        F: Fn(RequestId, Vec<u8>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.on_synthesising = Some(Arc::new(Box::new(move |request_id, audio| {
            Box::pin(func(request_id, audio))
        })));
        self
    }

    pub fn on_audio_metadata<F, Fut>(mut self, func: F) -> Self
    where
        F: Fn(RequestId, Vec<message::Metadata>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.on_audio_metadata = Some(Arc::new(Box::new(move |request_id, metadata| {
            Box::pin(func(request_id, metadata))
        })));
        self
    }

    pub fn on_synthesised<F, Fut>(mut self, func: F) -> Self
    where
        F: Fn(RequestId) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.on_synthesised = Some(Arc::new(Box::new(move |request_id| {
            Box::pin(func(request_id))
        })));
        self
    }
}
#[async_trait::async_trait]
impl crate::callback::Callback for Callback {
    type Item = crate::Result<Event>;

    #[allow(clippy::manual_async_fn)]
    fn on_event(&self, item: Self::Item) -> impl Future<Output = ()> {
        async move {
            match &item {
                Ok(Event::SessionStarted(request_id)) => {
                    tracing::debug!("Session started");
                    if let Some(f) = self.on_session_started.as_ref() {
                        f(*request_id).await
                    }
                }
                Ok(Event::SessionEnded(request_id)) => {
                    tracing::debug!("Session ended");
                    if let Some(f) = self.on_session_ended.as_ref() {
                        f(*request_id).await
                    }
                }

                Ok(Event::Synthesising(request_id, audio)) => {
                    tracing::debug!("Synthesising audio: {:?}", audio.len());
                    if let Some(f) = self.on_synthesising.as_ref() {
                        f(*request_id, audio.clone()).await
                    }
                }

                Ok(Event::Synthesised(request_id)) => {
                    tracing::debug!("Synthesised");
                    if let Some(f) = self.on_synthesised.as_ref() {
                        f(*request_id).await
                    }
                }

                Ok(Event::AudioMetadata(request_id, metadata)) => {
                    tracing::debug!("Audio metadata: {:?}", metadata);
                    if let Some(f) = self.on_audio_metadata.as_ref() {
                        f(*request_id, metadata.clone()).await
                    }
                }

                Err(e) => {
                    tracing::error!("Error: {:?}", e);
                    if let Some(_f) = self.on_error.as_ref() {
                        //f(session3.request_id(), e.clone()).await
                    }
                }
            }
        }
    }
}
