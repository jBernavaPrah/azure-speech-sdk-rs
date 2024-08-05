use crate::recognizer::Offset;

#[derive(Debug, Default, Clone)]
pub(crate) struct Session {
    pub(crate) request_id: uuid::Uuid,
    pub(crate) stream_id: Option<String>,
    pub(crate) webrtc_connection_string: Option<String>,
    pub(crate) bytes_received: usize,
    pub(crate) text_offset: Offset,
    pub(crate) next_search_text_index: usize,
    pub(crate) sentence_offset: Offset,
    pub(crate) next_search_sentence_index: usize,
    pub(crate) partial_viseme_animation: String,
}

impl Session {
    pub(crate) fn new(uuid: uuid::Uuid) -> Self {
        Self {
            request_id: uuid,
            ..Default::default()
        }
    }
}