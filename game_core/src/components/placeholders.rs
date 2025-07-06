//! Placeholder components for types that will be moved during domain separation

use bevy::prelude::*;

/// Placeholder for UnifiedCullable - will be moved to gameplay_render
#[derive(Component, Default)]
pub struct UnifiedCullable;

/// Placeholder for MovementTracker - will be moved to gameplay_sim  
#[derive(Component, Default)]
pub struct MovementTracker;

/// Placeholder for UnifiedChunkEntity - will be moved to gameplay_sim
#[derive(Component, Default)]
pub struct UnifiedChunkEntity;
