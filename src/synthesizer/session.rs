#[allow(dead_code)]
use crate::recognizer::Offset;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct SessionExt {
    request_id: uuid::Uuid,
    stream_id: Option<String>,
    webrtc_connection_string: Option<String>,
    #[allow(dead_code)]
    bytes_received: usize,
    #[allow(dead_code)]
    text_offset: Offset,
    #[allow(dead_code)]
    next_search_text_index: usize,
    #[allow(dead_code)]
    sentence_offset: Offset,
    #[allow(dead_code)]
    next_search_sentence_index: usize,
    #[allow(dead_code)]
    partial_viseme_animation: String,
}

#[derive(Clone)]
pub(crate) struct Session {
    inner: Arc<Mutex<SessionExt>>,
}

impl Session {
    pub(crate) fn new(uuid: uuid::Uuid) -> Self {
        Self {
            inner: Arc::new(Mutex::new(SessionExt {
                request_id: uuid,
                ..Default::default()
            })),
        }
    }

    pub(crate) fn request_id(&self) -> uuid::Uuid {
        self.inner.lock().unwrap().request_id
    }

    pub(crate) fn set_stream_id(&self, stream_id: String) {
        self.inner.lock().unwrap().stream_id = Some(stream_id);
    }

    pub(crate) fn stream_id(&self) -> Option<String> {
        self.inner.lock().unwrap().stream_id.clone()
    }

    pub(crate) fn set_webrtc_connection_string(&self, webrtc_connection_string: String) {
        self.inner.lock().unwrap().webrtc_connection_string = Some(webrtc_connection_string);
    }

    #[allow(dead_code)]
    pub(crate) fn webrtc_connection_string(&self) -> Option<String> {
        self.inner.lock().unwrap().webrtc_connection_string.clone()
    }

    #[allow(dead_code)]
    pub(crate) fn set_bytes_received(&self, bytes_received: usize) {
        self.inner.lock().unwrap().bytes_received = bytes_received;
    }

    #[allow(dead_code)]
    pub(crate) fn bytes_received(&self) -> usize {
        self.inner.lock().unwrap().bytes_received
    }

    #[allow(dead_code)]
    pub(crate) fn set_text_offset(&self, text_offset: Offset) {
        self.inner.lock().unwrap().text_offset = text_offset;
    }

    #[allow(dead_code)]
    pub(crate) fn text_offset(&self) -> Offset {
        self.inner.lock().unwrap().text_offset
    }

    #[allow(dead_code)]
    pub(crate) fn set_next_search_text_index(&self, next_search_text_index: usize) {
        self.inner.lock().unwrap().next_search_text_index = next_search_text_index;
    }

    #[allow(dead_code)]
    pub(crate) fn next_search_text_index(&self) -> usize {
        self.inner.lock().unwrap().next_search_text_index
    }

    #[allow(dead_code)]
    pub(crate) fn set_sentence_offset(&self, sentence_offset: Offset) {
        self.inner.lock().unwrap().sentence_offset = sentence_offset;
    }

    #[allow(dead_code)]
    pub(crate) fn sentence_offset(&self) -> Offset {
        self.inner.lock().unwrap().sentence_offset
    }

    #[allow(dead_code)]
    pub(crate) fn set_next_search_sentence_index(&self, next_search_sentence_index: usize) {
        self.inner.lock().unwrap().next_search_sentence_index = next_search_sentence_index;
    }

    #[allow(dead_code)]
    pub(crate) fn next_search_sentence_index(&self) -> usize {
        self.inner.lock().unwrap().next_search_sentence_index
    }

    #[allow(dead_code)]
    pub(crate) fn set_partial_viseme_animation(&self, partial_viseme_animation: String) {
        self.inner.lock().unwrap().partial_viseme_animation = partial_viseme_animation;
    }

    #[allow(dead_code)]
    pub(crate) fn partial_viseme_animation(&self) -> String {
        self.inner.lock().unwrap().partial_viseme_animation.clone()
    }
}
