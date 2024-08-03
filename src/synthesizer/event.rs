use serde::{Deserialize};

#[derive(Debug, Clone, Deserialize, PartialEq)]
/// Event for the speech recognition
pub enum Event {
    SessionStarted,
    SessionEnded,
    
    AudioMetadata(String),

    /// Raw Audio Chunk from the synthesizer.
    Synthesising(Vec<u8>),
    /// Synthesizing has finished.
    Synthesised,
}
