# Azure Speech SDK - Rust Implementation (Unofficial)

This is an unofficial, pure Rust library for working with the Azure Speech service. It is currently under development and not yet ready for production use. The library does not rely on any external dependencies.

This project is heavily inspired by the official Azure Speech SDK for Typescript (https://github.com/microsoft/cognitive-services-speech-sdk-js). The goal is to provide an easy-to-install and simple-to-use interface for working with the Azure Speech service.

## Why This Library?

The motivation behind creating this library was twofold:

1. To gain a deeper understanding of Rust.
2. To provide a Rust implementation of the Azure Speech SDK that doesn't require the C++ SDK as a dependency and is less complex than existing implementations (see https://github.com/jabber-tools/cognitive-services-speech-sdk-rs for reference).

## Installation

As the library is not yet published on crates.io, you can install it by adding the following to your `Cargo.toml` file:

```toml
[dependencies]
azure_speech = { version = "^0" }
```

## Usage
For usage examples, please refer to the examples folder in the repository.  

## Contributing
We welcome contributions! Feel free to submit PRs and raise issues. Your feedback and contributions can help shape the development of this library.  

## Support
If you find this project useful, please consider giving it a star on GitHub. 

This helps us know that people are interested and appreciate the work we're doing.  

Please note that this library is still a work in progress. We recommend using the Rust Azure Speech SDK (https://github.com/jabber-tools/cognitive-services-speech-sdk-rs) for production use in the meantime. 

Thank you for your interest in our project!