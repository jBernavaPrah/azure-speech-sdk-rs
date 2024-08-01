#![deny(unsafe_code)]

//! Azure Speech SDK - Pure Rust, unofficial and opinionated project.
//!
//! This crate provides a high-level API to interact with Azure Speech Services.
//! It is designed to be simple and easy to use, while providing a lot of flexibility,
//! without any external C dependencies, and it is based on the `tokio` runtime.
//!
//! It's use channels to return the `events`. The `events` are the result of the recognition process.
//!
//! This crate not require any external C dependencies, and it is based on the `tokio` runtime.
//!
//! For more information about Microsoft Speech Service see [here](https://docs.microsoft.com/en-us/azure/cognitive-services/speech-service/speech-sdk?tabs=windows%2Cubuntu%2Cios-xcode%2Cmac-xcode%2Candroid-studio).
//!
//! # Features
//! - [X] Speech to Text
//!     - [X] Continuous Recognition
//!     - [X] Single Shot Recognition (with a manual break in the events loop)
//!     - [X] File Recognition (with hound crate)
//!     - [X] Microphone Recognition (with cpal crate)
//!     - [X] Stream Recognition (with tokio::sync::mpsc)
//!     - [ ] Translation (work in progress) 
//! - [ ] Text to Speech (work in progress)
//!
//!
//! # Example
//! You can find examples in the `examples` directory.
//!

/// Specific events for the speech recognition
//mod recognizer;
mod connector;
mod auth;
mod utils;
mod config;
mod error;
mod stream_ext;

pub mod synthesizer;
mod iterator_ext;

pub use auth::*;
pub use connector::{Message, Data};
pub use error::{Error, Result};
pub use stream_ext::StreamExt;
