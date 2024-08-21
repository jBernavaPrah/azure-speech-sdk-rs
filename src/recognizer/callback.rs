use crate::callback::{BoxFuture, OnError, OnSessionEnded, OnSessionStarted};
use crate::recognizer::{Duration, Event, Offset, RawMessage, Recognized};
use crate::RequestId;
use std::future::Future;
use std::sync::Arc;

pub(crate) type OnRecognizing =
    Box<dyn Fn(RequestId, Recognized, Offset, Duration, RawMessage) -> BoxFuture>;
pub(crate) type OnRecognized =
    Box<dyn Fn(RequestId, Recognized, Offset, Duration, RawMessage) -> BoxFuture>;
pub(crate) type OnUnMatch = Box<dyn Fn(RequestId, Offset, Duration, RawMessage) -> BoxFuture>;
pub(crate) type OnStartDetected = Box<dyn Fn(RequestId, Offset) -> BoxFuture>;
pub(crate) type OnEndDetected = Box<dyn Fn(RequestId, Offset) -> BoxFuture>;

#[derive(Default, Clone)]
pub struct Callback {
    pub(crate) on_session_started: Option<Arc<OnSessionStarted>>,
    pub(crate) on_error: Option<Arc<OnError>>,
    pub(crate) on_session_ended: Option<Arc<OnSessionEnded>>,

    pub(crate) on_recognizing: Option<Arc<OnRecognizing>>,
    pub(crate) on_recognized: Option<Arc<OnRecognized>>,
    pub(crate) on_un_match: Option<Arc<OnUnMatch>>,
    pub(crate) on_start_detected: Option<Arc<OnStartDetected>>,
    pub(crate) on_end_detected: Option<Arc<OnEndDetected>>,
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
        self.on_session_ended = Some(Arc::new(Box::new(move |str| Box::pin(func(str)))));
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

    pub fn on_recognizing<F, Fut>(mut self, func: F) -> Self
    where
        F: Fn(RequestId, Recognized, Offset, Duration, RawMessage) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.on_recognizing = Some(Arc::new(Box::new(
            move |request_id, recognized, offset, duration, raw_message| {
                Box::pin(func(request_id, recognized, offset, duration, raw_message))
            },
        )));
        self
    }

    pub fn on_recognized<F, Fut>(mut self, func: F) -> Self
    where
        F: Fn(RequestId, Recognized, Offset, Duration, RawMessage) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.on_recognized = Some(Arc::new(Box::new(
            move |request_id, recognized, offset, duration, raw_message| {
                Box::pin(func(request_id, recognized, offset, duration, raw_message))
            },
        )));
        self
    }

    pub fn on_un_match<F, Fut>(mut self, func: F) -> Self
    where
        F: Fn(RequestId, Offset, Duration, RawMessage) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.on_un_match = Some(Arc::new(Box::new(
            move |request_id, offset, duration, raw_message| {
                Box::pin(func(request_id, offset, duration, raw_message))
            },
        )));
        self
    }

    pub fn on_start_detected<F, Fut>(mut self, func: F) -> Self
    where
        F: Fn(RequestId, Offset) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.on_start_detected = Some(Arc::new(Box::new(move |request, offset| {
            Box::pin(func(request, offset))
        })));
        self
    }

    pub fn on_end_detected<F, Fut>(mut self, func: F) -> Self
    where
        F: Fn(RequestId, Offset) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.on_end_detected = Some(Arc::new(Box::new(move |request, offset| {
            Box::pin(func(request, offset))
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

                Ok(Event::Recognizing(request_id, recognized, offset, duration, raw)) => {
                    if let Some(f) = self.on_recognizing.as_ref() {
                        f(
                            *request_id,
                            recognized.clone(),
                            *offset,
                            *duration,
                            raw.clone(),
                        )
                        .await
                    }
                }

                Ok(Event::Recognized(request_id, recognized, offset, duration, raw)) => {
                    if let Some(f) = self.on_recognized.as_ref() {
                        f(
                            *request_id,
                            recognized.clone(),
                            *offset,
                            *duration,
                            raw.clone(),
                        )
                        .await
                    }
                }

                Ok(Event::UnMatch(request_id, offset, duration, raw)) => {
                    if let Some(f) = self.on_un_match.as_ref() {
                        f(*request_id, *offset, *duration, raw.clone()).await
                    }
                }

                Ok(Event::EndDetected(request_id, offset)) => {
                    if let Some(f) = self.on_end_detected.as_ref() {
                        f(*request_id, *offset).await
                    }
                }

                Ok(Event::StartDetected(request_id, offset)) => {
                    if let Some(f) = self.on_start_detected.as_ref() {
                        f(*request_id, *offset).await
                    }
                }

                Err(e) => {
                    tracing::error!("Error: {:?}", e);
                    if let Some(_f) = self.on_error.as_ref() {
                        // todo: improve the error with adding the request_id on it!
                        // f(session.request_id(), e.clone())
                    }
                }
            }
        }
    }
}
