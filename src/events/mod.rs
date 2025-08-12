//! Event-driven communication system for cross-plugin coordination
//! 
//! This module provides lightweight events (≤128 bytes, Copy/Clone) that enable
//! decoupled communication between plugins while maintaining explicit data flow.
//! 
//! Each event group is organized in its own module with clear purpose documentation.

pub mod world;
pub mod distance_events;
pub mod ground_events;
pub mod player_events;
mod size_verification;

#[cfg(feature = "debug-ui")]
pub mod debug_instrumentation;

// Re-export all world generation events for convenience
pub use world::{
    chunk_events::*,
    content_events::*,
    validation_events::*,
};

// Re-export service coordination events
pub use distance_events::*;
pub use ground_events::*;
pub use player_events::*;

#[cfg(feature = "debug-ui")]
pub use debug_instrumentation::*;
