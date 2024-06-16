# Azure Speech SDK - Rust

This is pure Rust library for working with the Azure Speech service.

The goal is to provide an easy-to-install and simple-to-use interface for working with the Azure Speech service.

This project is inspired by the official Azure Speech SDK for Javascript (https://github.com/microsoft/cognitive-services-speech-sdk-js).

**It is currently under development and not yet ready for production use.**
I recommend using the Rust Azure Speech SDK (https://github.com/jabber-tools/cognitive-services-speech-sdk-rs) for production use in the meantime.

## Why This Library?

The motivation behind creating this library was twofold:

1. To gain a deeper understanding of Rust.
2. To provide a Rust implementation of the Azure Speech SDK that doesn't require the C++ SDK as a dependency and is less complex than existing implementations (see https://github.com/jabber-tools/cognitive-services-speech-sdk-rs for reference).

## Installation

Use the following command to add the library to your project:

```bash
  cargo add azure_speech
```

Add the following to your `Cargo.toml` file:
```toml
[dependencies]
# ....
azure_speech = { version = "0.1" }
```

## Usage
For usage examples, please refer to the examples folder in the repository.  

## Contributing
Feel free to submit PRs and raise issues. Your feedback and contributions can help shape the development of this library.  

## Support
If you find this project useful, please consider giving it a star on GitHub. 

This helps me know that people are interested.  
