// Pure utilities and math - no Bevy dependencies
#![deny(clippy::all, clippy::pedantic)]
#![warn(missing_docs)]

//! Core engine utilities and math functions
//! This crate contains pure Rust code with no external dependencies

pub mod math;
pub mod utils;
pub mod timing;
pub mod performance;
pub mod prelude;

// Only expose via prelude - no direct re-exports
