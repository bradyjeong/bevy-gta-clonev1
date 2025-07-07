// Pure utilities and math - no Bevy dependencies
#![deny(clippy::all, clippy::pedantic)]
#![deny(missing_docs)]

//! Core engine utilities and math functions
//! This crate contains pure Rust code with no external dependencies

#[allow(missing_docs)]
pub(crate) mod math;
#[allow(missing_docs)]
pub(crate) mod utils;
#[allow(missing_docs)]
pub(crate) mod timing;
#[allow(missing_docs)]
pub(crate) mod performance;
pub mod prelude;

// Only expose via prelude - no direct re-exports
