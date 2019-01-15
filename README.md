# streamdeck-rs

> Unofficial [Stream Deck](https://www.elgato.com/en/gaming/stream-deck) SDK for Rust

[![Crates.io](https://img.shields.io/crates/v/streamdeck-rs.svg)](https://crates.io/crates/streamdeck-rs) ![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg) [![Build status](https://travis-ci.org/mdonoughe/streamdeck-rs.svg)](https://travis-ci.org/mdonoughe/streamdeck-rs/) [![Docs.rs](https://docs.rs/streamdeck-rs/badge.svg)](https://docs.rs/streamdeck-rs)

Elgato's official [Stream Deck SDK](https://developer.elgato.com/documentation/stream-deck/sdk/overview/) works by launching plugins in their own processes and communicating via web sockets. This library provides the command line argument parsing and basic protocol details for creating a plugin using Rust.

This library is pretty basic for now. In the future it could provide a framework for instancing actions (keys) and routing messages to the appropriate instances.
