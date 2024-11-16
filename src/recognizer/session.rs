use crate::recognizer::Offset;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default, Clone)]
struct SessionInner {
    request_id: uuid::Uuid,
    is_audio_completed: bool,
    audio_offset: Offset,
    recognition_offset: Offset,
    hypothesis_received: bool,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct Session {
    inner: Arc<Mutex<SessionInner>>,
}

impl Session {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(SessionInner {
                request_id: uuid::Uuid::new_v4(),
                ..Default::default()
            })),
        }
    }

    pub(crate) fn refresh(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.request_id = uuid::Uuid::new_v4();
        inner.is_audio_completed = false;
        inner.audio_offset = 0;
        inner.recognition_offset = 0;
        inner.hypothesis_received = false;
    }

    pub(crate) fn on_hypothesis_received(&self, _offset: Offset) {
        self.inner.lock().unwrap().hypothesis_received = true;
        // todo: update telemetry..
    }

    pub(crate) fn on_phrase_recognized(&self, offset: Offset) {
        let mut inner = self.inner.lock().unwrap();
        inner.recognition_offset += offset;
        inner.hypothesis_received = false;
    }

    pub(crate) fn is_audio_completed(&self) -> bool {
        self.inner.lock().unwrap().is_audio_completed
    }

    pub(crate) fn set_audio_completed(&self, is_audio_completed: bool) {
        self.inner.lock().unwrap().is_audio_completed = is_audio_completed;
    }

    pub(crate) fn request_id(&self) -> uuid::Uuid {
        self.inner.lock().unwrap().request_id
    }

    pub(crate) fn audio_offset(&self) -> Offset {
        self.inner.lock().unwrap().audio_offset
    }

    #[allow(dead_code)]
    pub(crate) fn set_audio_offset(&self, audio_offset: Offset) {
        self.inner.lock().unwrap().audio_offset = audio_offset;
    }

    #[allow(dead_code)]
    pub(crate) fn recognition_offset(&self) -> Offset {
        self.inner.lock().unwrap().recognition_offset
    }

    #[allow(dead_code)]
    pub(crate) fn set_recognition_offset(&self, recognition_offset: Offset) {
        self.inner.lock().unwrap().recognition_offset = recognition_offset;
    }
}
