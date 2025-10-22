use bevy_rapier3d::prelude::*;

// Collision groups for proper physics separation
pub const STATIC_GROUP: Group = Group::GROUP_1; // Buildings, terrain, trees
pub const VEHICLE_GROUP: Group = Group::GROUP_2; // Cars, helicopters, jets
pub const CHARACTER_GROUP: Group = Group::GROUP_3; // Player, NPCs

// World elevation constants
pub const SEA_LEVEL: f32 = 0.0; // Ocean surface height (animated water level)
pub const LAND_ELEVATION: f32 = 3.0; // Terrain height above sea level
pub const SPAWN_DROP_HEIGHT: f32 = 10.0; // Height above terrain to spawn entities for gravity drop
pub const OCEAN_FLOOR_DEPTH: f32 = -10.0; // Ocean floor Y position

// Rectangular island configuration (triple island setup)
pub const LEFT_ISLAND_X: f32 = -1500.0; // Left island center X position
pub const RIGHT_ISLAND_X: f32 = 1500.0; // Right island center X position
pub const GRID_ISLAND_X: f32 = 0.0; // Grid island center X position
pub const GRID_ISLAND_Z: f32 = 1800.0; // Grid island center Z position
pub const TERRAIN_SIZE: f32 = 1200.0; // Terrain square size (1200m x 1200m)
pub const TERRAIN_HALF_SIZE: f32 = TERRAIN_SIZE / 2.0; // 600m from center to edge
pub const BEACH_WIDTH: f32 = 100.0; // Beach slope width extending from terrain edge

// Island boundaries for validation
// Left island: X ∈ [-2100, -900], Z ∈ [-600, 600]
// Right island: X ∈ [900, 2100], Z ∈ [-600, 600]
// Grid island: X ∈ [-600, 600], Z ∈ [1200, 2400]

// World boundary failsafe (prevents falling into void)
pub const MAX_WORLD_COORDINATE: f32 = 3000.0; // Extreme boundary for safety teleport
