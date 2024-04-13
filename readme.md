
# Work in progress
This library is still in development and is not ready for production use.  
I suggest using the Rust Azure Speech SDK (https://github.com/jabber-tools/cognitive-services-speech-sdk-rs) for now.

### PRs and issues are welcome!
### Add a star if you like the project, will help me know if people are interested in this project.

# Azure Speech SDK
An **Unofficial** Rust library for working with the Azure Speech service **without** external dependencies.

Heavily inspired by the official Azure Speech SDK for Typescript (https://github.com/microsoft/cognitive-services-speech-sdk-js).

The goal of this library is to provide a simple installation and interface for working with the Azure Speech service.

## Motivation
There were 2 main reasons for creating this library: 
1. I wanted to learn better in Rust.
2. The only Rust Azure Speech SDK (https://github.com/jabber-tools/cognitive-services-speech-sdk-rs), uses the C++ SDK as a dependency. I wanted to create a library that does not have any external dependencies.

## Installation
For now the library is not published on crates.io. You can install it by adding the following to your `Cargo.toml` file:
```toml
[dependencies]
azure_speech = { git = "https://github.com/jBernavaPrah/azure-speech.git" }
```

## Usage
Check the examples folder for more examples.




