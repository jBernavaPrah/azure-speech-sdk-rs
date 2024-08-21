# Azure Speech SDK - Rust

![Crates.io License](https://img.shields.io/crates/l/azure-speech)
![GitHub License](https://img.shields.io/github/license/jBernavaPrah/azure-speech-sdk-rs)
![Crates.io Downloads (recent)](https://img.shields.io/crates/dr/azure-speech)
![Crates.io Version](https://img.shields.io/crates/v/azure-speech)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/jBernavaPrah/azure-speech-sdk-rs/build.yml)
![GitHub Issues](https://img.shields.io/github/issues/jBernavaPrah/azure-speech-sdk-rs)
![GitHub Pull Requests](https://img.shields.io/github/issues-pr/jBernavaPrah/azure-speech-sdk-rs)
![GitHub Last Commit](https://img.shields.io/github/last-commit/jBernavaPrah/azure-speech-sdk-rs)
![GitHub Contributors](https://img.shields.io/github/contributors/jBernavaPrah/azure-speech-sdk-rs)


Welcome to the Azure Speech SDK, a pure Rust library for interacting with Azure Speech Services.

## Overview

This library aims to provide an easy-to-install and straightforward interface for working with Azure Speech Services. Inspired by the official [Azure Speech SDK for JavaScript](https://github.com/microsoft/cognitive-services-speech-sdk-js), it is designed to be simple and user-friendly.

The library currently supports the following features:

- [X] Speech Recognition (Speech-to-Text) [examples](examples/recognize_simple.rs)
  - [X] Real-time Speech Recognition
  - [X] Custom Speech Recognition
  - [X] Phrase List
  - [ ] Conversation Transcriber - Real-time Diarization (Work in Progress)
  - [ ] Pronunciation Assessment (Work in Progress)
- [X] Speech Synthesis (Text-to-Speech) [example](examples/synthesize_simple.rs)
  - [X] Real-time Speech Synthesis
  - [X] Custom Voice
  - [X] SSML Support
- [ ] Speech Translation (Work in Progress)
- [ ] Intent Recognition (Work in Progress)
- [ ] Keyword Recognition (Work in Progress)

The library is currently in the early stages of development, and I am actively working on adding more features and improving the existing ones.

## Why This Library?

The motivation behind creating this library includes:

1. Providing a Rust implementation of the Azure Speech SDK that eliminates the need for the C++ SDK dependency, offering a simpler alternative to existing implementations.
2. Gaining a deeper understanding of Rust. This project serves as a learning experience for me to explore Rust's capabilities and best practices.

## Installation

Add this library to your project using the following command:

```bash
cargo add azure_speech
```

**And that's it!** 

You are now ready to use the Azure Speech SDK in your Rust project.

## Usage
For usage examples, please refer to the [examples folder](https://github.com/jBernavaPrah/azure-speech-sdk-rs/tree/master/examples) in the repository. Or check the [documentation](https://docs.rs/azure-speech).

## Contributing
We welcome contributions! Feel free to submit pull requests and raise issues. Your feedback and contributions are invaluable in shaping the development of this library.

## Support
If you find this project useful, please consider giving it a subscribe on GitHub. Your support is greatly appreciated.