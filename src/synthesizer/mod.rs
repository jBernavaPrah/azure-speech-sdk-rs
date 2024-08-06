//! Text to speech module.
//! 
//! This module provides a client to interact with the Azure Text to Speech service.
//! 
//! # Example
//! 
//! ```
//!use azure_speech::{synthesizer, Auth, stream::StreamExt};
//! use std::env;
//! use std::error::Error;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     tracing_subscriber::fmt()
//!         .with_max_level(tracing::Level::DEBUG)
//!         .init();
//!     // Add your Azure region and subscription key to the environment variables
//!     let auth = Auth::from_subscription(
//!             env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
//!             env::var("AZURE_SUBSCRIPTION_KEY")
//!                 .expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
//!         );
//! 
//!     // Set auth and the configuration for the synthesizer
//!     let client = synthesizer::Client::connect(auth,synthesizer::Config::default()).await.expect("to connect to azure");
//!     let mut stream = client.synthesize("Hello world!").await.expect("to synthesize");
//! 
//!     while let Some(event) = stream.next().await {
//!         match event {
//!              _ => tracing::info!("Event: {:?}", event)
//!         }
//!     } 
//!     Ok(())
//! }
//! 
//! ```

mod audio_format;
mod client;
mod config;
mod event;
mod message;
mod session;
mod ssml;
mod utils;

pub use audio_format::*;
pub use client::*;
pub use config::*;
pub use event::*;
pub use ssml::*;
