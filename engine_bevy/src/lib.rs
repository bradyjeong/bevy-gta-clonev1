//! Bevy engine abstractions and utilities
#![deny(clippy::all, clippy::pedantic)]
#![deny(missing_docs)]

use bevy::prelude::*;
pub use engine_core;

pub(crate) mod adapters;
pub mod prelude;
pub(crate) mod services;

// Only expose via prelude - no direct re-exports
