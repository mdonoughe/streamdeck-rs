# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2019-06-15
### Added
- New events and properties from Stream Deck software version 4.3.

### Changed
- Unrecognized events are now reported as `Message::Unknown` instead of an error.

## [0.2.0] - 2019-03-10
### Added
- Logging support via Slog.

### Changed
- Now for SDK version 2. You must update the plugin manifest file. See https://developer.elgato.com/documentation/stream-deck/sdk/changelog/#changes-in-stream-deck-41

## 0.1.0 - 2019-01-14
### Added
- Command line parsing.
- Protocol for registration and message handling.

[Unreleased]: https://github.com/mdonoughe/streamdeck-rs/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/mdonoughe/streamdeck-rs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/mdonoughe/streamdeck-rs/compare/v0.1.0...v0.2.0
