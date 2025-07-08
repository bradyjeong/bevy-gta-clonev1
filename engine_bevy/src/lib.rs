//! Bevy engine abstractions and utilities
#![deny(warnings)]
#![deny(clippy::all, clippy::pedantic)]
#![deny(missing_docs)]

pub use engine_core;

pub(crate) mod adapters;
pub mod prelude;
pub mod services;

// Only expose via prelude - no direct re-exports
