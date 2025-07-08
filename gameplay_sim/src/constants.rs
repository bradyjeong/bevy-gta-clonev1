//! Constants for gameplay simulation

use bevy_rapier3d::geometry::Group;

/// Collision groups
pub const STATIC_GROUP: Group = Group::GROUP_1;
pub const VEHICLE_GROUP: Group = Group::GROUP_2;
pub const CHARACTER_GROUP: Group = Group::GROUP_3;

/// Common distances
pub const CHUNK_SIZE: f32 = 64.0;
pub const UNIFIED_CHUNK_SIZE: f32 = 128.0;
pub const UNIFIED_STREAMING_RADIUS: f32 = 300.0;

/// Performance constants
pub const MAX_ENTITIES_PER_CHUNK: usize = 100;
pub const DISTANCE_CACHE_SIZE: usize = 2048;
