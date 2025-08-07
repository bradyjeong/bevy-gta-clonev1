//! Shared types and utilities to break circular dependencies
//! 
//! This module contains types that are shared between different modules
//! to avoid circular dependencies between systems and factories.

pub mod movement_tracking;
pub mod world_types;

pub use movement_tracking::*;
pub use world_types::*;
