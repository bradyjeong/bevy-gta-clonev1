//! Bevy engine abstractions and utilities
#![warn(missing_docs)]

use bevy::prelude::*;
pub use engine_core;

pub mod adapters;
pub mod prelude;
pub mod services;

pub use adapters::*;
pub use prelude::*;
