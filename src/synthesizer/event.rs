//! Event for the speech recognition
//!     

use crate::synthesizer::message;
use crate::RequestId;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Event for the speech recognition
pub enum Event {
    SessionStarted(RequestId),
    SessionEnded(RequestId),

    AudioMetadata(RequestId, Vec<message::Metadata>),
    /// Raw Audio Chunk from the synthesizer.
    Synthesising(RequestId, Vec<u8>),
    /// Synthesizing has finished.
    Synthesised(RequestId),
}
