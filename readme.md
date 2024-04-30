
# Work in progress
This library is still in development and is not ready for production use.  
In the meantime I suggest using the Rust Azure Speech SDK (https://github.com/jabber-tools/cognitive-services-speech-sdk-rs).

### PRs and issues are welcome!
### Add a star if you like the project, will help me know if people are interested in this project.

# Azure Speech SDK
An **Unofficial** pure Rust library for working with the Azure Speech service **without** any external dependencies.

Heavily inspired by the official Azure Speech SDK for Typescript (https://github.com/microsoft/cognitive-services-speech-sdk-js).

The goal of this library is to provide a easy installation and simple interface for working with the Azure Speech service.

## Motivation
There were 2 main reasons for creating this library: 
1. I wanted to learn better in Rust.
2. The only Rust implementation of Azure Speech SDK (https://github.com/jabber-tools/cognitive-services-speech-sdk-rs), require the C++ SDK as a dependency and as being inspired by Go implementation, share the same complexity (see pullStream vs pushStream or microphone settings ecc..).

## Installation
For now the library is not published on crates.io. You can install it by adding the following to your `Cargo.toml` file:
```toml
[dependencies]
azure_speech = { git = "https://github.com/jBernavaPrah/azure-speech.git" }
```

## Usage
Check the examples folder for more examples.




