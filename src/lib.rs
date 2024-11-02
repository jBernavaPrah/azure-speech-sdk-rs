#![deny(unsafe_code)]

//! # Azure Speech SDK - Pure Rust Implementation
//!
//! Welcome to the Azure Speech SDK crate, an unofficial, opinionated Rust project.
//! This crate offers a high-level API to interact with Azure Speech Services, designed
//! for simplicity and flexibility without any external C dependencies. Built on the
//! `tokio` runtime, it minimizes external dependencies wherever possible.
//!
//! ## Core Functionalities
//! - [X] Speech to Text [recognizer]
//! - [X] Text to Speech [synthesizer]
//!
//! For comprehensive information on Microsoft Speech Service, refer to the official
//! documentation [here](https://learn.microsoft.com/en-us/azure/ai-services/speech-service/speech-sdk).
//!
//! ## Notes
//! This crate, in its current version, does not include some features available in the
//! official SDK, such as microphone/file recognition or synthesizer output to speakers.
//! However, examples demonstrating these capabilities can be found in the `examples` directory.
//!
//! ## Usage and Examples
//! Detailed usage instructions and examples are provided in the [examples](https://github.com/jBernavaPrah/azure-speech-sdk-rs/blob/master/examples) folder in the GitHub repository.
//!

mod auth;
mod config;
pub mod connector;
mod error;
mod event;
mod stream_ext;
mod utils;

mod callback;
pub mod recognizer;
pub mod synthesizer;

pub use auth::*;
pub use connector::*;
pub use error::*;

pub use event::*;
pub use stream_ext::*;

pub mod stream {
    //! Re-export of `tokio_stream` crate.
    pub use tokio_stream::*;
}
