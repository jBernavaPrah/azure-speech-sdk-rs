use serde::{Deserialize};

#[derive(Debug, Clone, Deserialize, PartialEq)]
/// Event for the speech recognition
pub enum EventSynthesis {
    SynthesisStarted,

    SynthesisCancelled(crate::Error),
    SynthesisCompleted,
    
    AudioMetadata(String),

    Synthesizing(Vec<u8>),
}
