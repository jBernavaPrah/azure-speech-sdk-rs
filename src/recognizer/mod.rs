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
pub async fn speech
(
    config: ResolverConfig,
    source: Source,
) -> crate::Result<Receiver<Event<speech::EventSpeech>>>
{
    let (_, event_rx) = recognize::<speech::EventSpeech>(config, source).await?;
    Ok(event_rx)
}
