use bevy_rapier3d::prelude::*;

// Collision groups for proper physics separation
pub const STATIC_GROUP: Group = Group::GROUP_1; // Buildings, terrain, trees
pub const VEHICLE_GROUP: Group = Group::GROUP_2; // Cars, helicopters, jets
pub const CHARACTER_GROUP: Group = Group::GROUP_3; // Player, NPCs

// World elevation constants
pub const SEA_LEVEL: f32 = 0.0; // Ocean surface height
pub const LAND_ELEVATION: f32 = 3.0; // Terrain height above sea level
pub const SPAWN_DROP_HEIGHT: f32 = 10.0; // Height above terrain to spawn entities for gravity drop
