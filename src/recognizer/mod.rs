//! Speech to text module.
//!
//! This module provides functionality to convert speech to text using Azure Speech Services.
//!
//! # Example
//!
//! ```compile_fail
//! use std::env;
//! use std::path::Path;
//! use tokio::fs::File;
//! use tokio::io::{AsyncReadExt, BufReader};
//!
//! use azure_speech::Auth;
//! use azure_speech::recognizer;
//! use azure_speech::stream::{Stream, StreamExt,wrappers::ReceiverStream};
//!
//! #[tokio::main]
//! async fn main() -> azure_speech::Result<()> {
//!
//!     let auth = Auth::from_subscription(
//!         env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
//!         env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
//!     );
//!
//!     let client = recognizer::Client::connect(auth, recognizer::Config::default()).await?;
//!
//!     // check in the example folder for how to create the audio stream.
//!     let audio_stream = create_audio_stream();
//!     let mut stream = client
//!         .recognize(audio_stream, recognizer::ContentType::Mp3, recognizer::Details::file())
//!         .await?;
//!
//!     while let Some(event) = stream.next().await {
//!         tracing::info!("Event: {:?}", event);
//!     }
//!
//!     tracing::info!("Completed!");
//!
//!     Ok(())
//! }
//!

mod client;
mod config;
mod content_type;
mod event;
mod language;
mod message;
mod session;
mod utils;
mod callback;

pub use client::*;
pub use config::*;
pub use content_type::*;
pub use event::*;
pub use language::*;
pub use callback::*;
