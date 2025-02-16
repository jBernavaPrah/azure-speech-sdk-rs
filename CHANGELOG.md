# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0](https://github.com/jBernavaPrah/azure-speech-sdk-rs/compare/v0.4.4...v0.5.0) - 2025-02-16

### Added

- Added Audio Metadata on the synthesizer client.
- Fix the content type for the recognizer. Wav, has already the headers in the file, vs raw, needs to specify the headers.

### Other

- Added comment on AudioFormat for the synthesizer.

## [0.4.4](https://github.com/jBernavaPrah/azure-speech-sdk-rs/compare/v0.4.3...v0.4.4) - 2025-02-11

### Fixed

- Improve disconnection from client.

## [0.4.3](https://github.com/jBernavaPrah/azure-speech-sdk-rs/compare/v0.4.2...v0.4.3) - 2025-01-27

### Other

- Fix clippy warning

## [0.4.2](https://github.com/jBernavaPrah/azure-speech-sdk-rs/compare/v0.4.1...v0.4.2) - 2024-12-21

### Fixed

- Remove the broadcast error

## [0.4.1](https://github.com/jBernavaPrah/azure-speech-sdk-rs/compare/v0.4.0...v0.4.1) - 2024-12-10

### Other

- Adding Debug to Config
- Merge remote-tracking branch 'origin/master'
- Minor improvements and bugfix.

## [0.4.0](https://github.com/jBernavaPrah/azure-speech-sdk-rs/compare/v0.3.2...v0.4.0) - 2024-11-16

### Other

- recognizer stream multi-turn ([#23](https://github.com/jBernavaPrah/azure-speech-sdk-rs/pull/23))

## [0.3.2](https://github.com/jBernavaPrah/azure-speech-sdk-rs/compare/v0.3.1...v0.3.2) - 2024-11-02

### Other

- Bugfix/broadcast sender fix ([#21](https://github.com/jBernavaPrah/azure-speech-sdk-rs/pull/21))

## [0.3.1](https://github.com/jBernavaPrah/azure-speech-sdk-rs/compare/v0.3.0...v0.3.1) - 2024-10-21

### Other

- development ([#19](https://github.com/jBernavaPrah/azure-speech-sdk-rs/pull/19))

## [0.3.0](https://github.com/jBernavaPrah/azure-speech-sdk-rs/compare/v0.2.3...v0.3.0) - 2024-09-21

### Added

- impl `std::error::Error` for Error

- Replace `ezsockets` with `tokio-websockets` ([#14](https://github.com/jBernavaPrah/azure-speech-sdk-rs/pull/14))

## [0.2.3](https://github.com/jBernavaPrah/azure-speech-sdk-rs/compare/v0.2.2...v0.2.3) - 2024-08-16

### Other
- Improve documentation  ([#10](https://github.com/jBernavaPrah/azure-speech-sdk-rs/pull/10))

## [0.2.2](https://github.com/jBernavaPrah/azure-speech-sdk-rs/compare/v0.2.1...v0.2.2) - 2024-08-16

### Other
- Merge Development to Master ([#8](https://github.com/jBernavaPrah/azure-speech-sdk-rs/pull/8))

## [0.2.1](https://github.com/jBernavaPrah/azure-speech-sdk-rs/compare/v0.2.0...v0.2.1) - 2024-08-07

### Other
- execute the build only during the pull request on the master ([#6](https://github.com/jBernavaPrah/azure-speech-sdk-rs/pull/6))

## [0.2.0](https://github.com/jBernavaPrah/azure-speech-sdk-rs/compare/v0.1.0...v0.2.0) - 2024-08-07

### Other
- Improve documentation and examples ([#4](https://github.com/jBernavaPrah/azure-speech-sdk-rs/pull/4))
- Improving examples
