# streamdeck-rs

> Unofficial [Stream Deck](https://www.elgato.com/en/gaming/stream-deck) SDK for Rust

[![Crates.io](https://img.shields.io/crates/v/streamdeck-rs.svg)](https://crates.io/crates/streamdeck-rs) ![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg) [![Build Status](https://github.com/mdonoughe/streamdeck-rs/actions/workflows/check.yml/badge.svg)](https://github.com/mdonoughe/streamdeck-rs/actions/workflows/check.yml) [![Docs.rs](https://docs.rs/streamdeck-rs/badge.svg)](https://docs.rs/streamdeck-rs)

Elgato's official [Stream Deck SDK](https://docs.elgato.com/sdk/plugins/overview) works by launching plugins in their own processes and communicating via web sockets. This library provides the command line argument parsing and basic protocol details for creating a plugin using Rust.

This library is pretty basic for now. In the future it could provide a framework for instancing actions (keys) and routing messages to the appropriate instances.

## Usage

1. Create a binary executable project.
2. Use `RegistrationParams` to get the information required to use `StreamDeckSocket`.
3. See [the official documentation](https://docs.elgato.com/sdk/plugins/overview) for information about [creating manifests](https://docs.elgato.com/sdk/plugins/manifest) and [loading your plugin](https://docs.elgato.com/sdk/plugins/getting-started).
