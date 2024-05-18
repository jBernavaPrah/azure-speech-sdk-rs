//! Speech to text module.

pub(crate) mod source;
mod event;
/// Specific events for the speech recognition
pub mod speech;
pub(crate) mod client;
pub(crate) mod utils;
/// Configuration for the speech recognition
pub mod config;

use tokio::sync::mpsc::Receiver;

pub use source::{Source, Details, WavSpec, SampleFormat, Sample};
pub use event::{Event, EventBase, CancelledReason};
use crate::recognizer::client::recognize;
use crate::recognizer::config::ResolverConfig;

/// Recognize the speech from the given source.
pub async fn speech<T: Sample>
(
    config: ResolverConfig,
    source: Source<T>,
) -> crate::errors::Result<Receiver<Event<speech::EventSpeech>>>
    where T: Sample {
    let (_, event_rx) = recognize::<speech::EventSpeech, T>(config, source).await?;
    Ok(event_rx)
}
