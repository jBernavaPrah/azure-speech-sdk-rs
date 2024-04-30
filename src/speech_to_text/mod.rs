mod recognizer;
pub(crate) mod source;
pub mod event;
pub mod speech;
pub(crate) mod client;
pub(crate) mod utils;
pub mod config;

// exports only the necessary types
pub use recognizer::{Recognizer};
pub use source::{Headers, AudioFormat, Source};
