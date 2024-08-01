use serde::{Deserialize};

#[derive(Debug, Clone, Deserialize, PartialEq)]
/// Event for the speech recognition
pub enum Event {
    Started,

    Cancelled(crate::Error),
    Completed,
    
    AudioMetadata(String),

    /// Raw Audio Chunk from the synthesizer.
    Audio(Vec<u8>),
}
