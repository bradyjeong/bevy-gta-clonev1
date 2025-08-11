//! Event-driven communication system for cross-plugin coordination
//! 
//! This module provides lightweight events (≤128 bytes, Copy/Clone) that enable
//! decoupled communication between plugins while maintaining explicit data flow.
//! 
//! Each event group is organized in its own module with clear purpose documentation.

pub mod world;
mod size_verification;

// Re-export all world generation events for convenience
pub use world::{
    chunk_events::*,
    content_events::*,
    validation_events::*,
};
