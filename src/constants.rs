use bevy_rapier3d::prelude::*;

// Collision groups for proper physics separation
pub const STATIC_GROUP: Group = Group::GROUP_1;    // Buildings, terrain, trees
pub const VEHICLE_GROUP: Group = Group::GROUP_2;   // Cars, helicopters, jets
pub const CHARACTER_GROUP: Group = Group::GROUP_3; // Player, NPCs

// World generation constants
/// Chunk size in world units (meters) - MUST be consistent across all systems
/// This re-exports UNIFIED_CHUNK_SIZE from world constants for backward compatibility
pub use crate::world::constants::UNIFIED_CHUNK_SIZE as CHUNK_SIZE;

/// Half chunk size for centering calculations
pub const HALF_CHUNK_SIZE: f32 = 100.0 * 0.5; // Must match UNIFIED_CHUNK_SIZE / 2

/// Content spawn radius within a chunk (in meters)
pub const CONTENT_SPAWN_RADIUS: f32 = 120.0; // Slightly less than half chunk
