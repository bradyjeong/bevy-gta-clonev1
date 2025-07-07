//! Test utilities for the GTA game codebase
#![deny(clippy::all, clippy::pedantic)]
#![deny(missing_docs)]

/// Minimal Bevy app for testing
pub mod minimal_app;
/// World helper utilities
pub mod world_helpers;
/// Screenshot testing utilities
pub mod screenshot;
/// Common testing prelude
pub mod prelude;
/// Golden frame testing utilities
pub mod golden_frame;

pub use minimal_app::MinimalBevyApp;
pub use world_helpers::*;
pub use screenshot::*;
pub use prelude::*;
pub use golden_frame::*;
