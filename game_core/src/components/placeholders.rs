//! Placeholder components for types that will be moved during domain separation
//! These now re-export the canonical spatial components

// Re-export the canonical spatial components for backwards compatibility
pub use super::spatial::{UnifiedCullable, MovementTracker, UnifiedChunkEntity};
