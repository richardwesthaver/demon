//! Demon CLI Module - sys/cli
//!
//! Core components for building Command-Line Interfaces. This crate
//! makes use of [termion](https://crates.io/crates/termion),
//! [tokio](http://crates.io/crates/tokio), and other core demon
//! modules.
//!
//! Eventually all binaries provided in bin/ will provide a CLI using
//! these components.
mod cons;
pub use cons::{DMC_BANNER, DMZ_BANNER};
pub mod app;
pub mod ui;
pub use cursive;

use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum CliError {}
