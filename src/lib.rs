#![deny(unsafe_code)]

//! # Azure Speech SDK - Pure Rust Implementation
//!
//! Welcome to the Azure Speech SDK crate, an unofficial, opinionated Rust project.
//! This crate offers a high-level API to interact with Azure Speech Services, designed
//! for simplicity and flexibility without any external C dependencies. Built on the
//! `tokio` runtime, it minimizes external dependencies wherever possible.
//!
//! ## Core Functionalities
//! - [X] Speech to Text
//! - [X] Text to Speech
//!
//! For comprehensive information on Microsoft Speech Service, refer to the official
//! documentation [here](https://docs.microsoft.com/en-us/azure/cognitive-services/speech-service/speech-sdk?tabs=windows%2Cubuntu%2Cios-xcode%2Cmac-xcode%2Candroid-studio).
//!
//! ## Notes
//! This crate, in its current version, does not include some features available in the
//! official SDK, such as microphone/file recognition or synthesizer output to speakers.
//! However, examples demonstrating these capabilities can be found in the `examples` directory.
//!
//! ## Usage and Examples
//! Detailed usage instructions and examples are provided in the `examples` directory.
//!

mod auth;
mod config;
mod connector;
mod error;
mod event;
mod stream_ext;
mod utils;

pub mod recognizer;
pub mod synthesizer;

pub use auth::*;
pub use connector::*;
pub use error::*;

pub use event::*;
pub use stream_ext::*;

pub mod stream {
    pub use tokio_stream::*;
}
