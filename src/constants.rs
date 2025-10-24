use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use serde::Deserialize;

// Collision groups for proper physics separation (implementation constant - not data-driven)
pub const STATIC_GROUP: Group = Group::GROUP_1; // Buildings, terrain, trees
pub const VEHICLE_GROUP: Group = Group::GROUP_2; // Cars, helicopters, jets
pub const CHARACTER_GROUP: Group = Group::GROUP_3; // Player, NPCs

// World environment configuration loaded from assets/config/world_config.ron
// Renamed to WorldEnvConfig to avoid collision with config::WorldConfig
#[derive(Debug, Clone, Resource, Deserialize)]
pub struct WorldEnvConfig {
    pub sea_level: f32,
    pub land_elevation: f32,
    pub spawn_drop_height: f32,
    pub ocean_floor_depth: f32,
    pub islands: IslandConfig,
    pub terrain: TerrainConfig,
    pub max_world_coordinate: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IslandConfig {
    pub left_x: f32,
    pub right_x: f32,
    pub grid_x: f32,
    pub grid_z: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TerrainConfig {
    pub size: f32,
    pub half_size: f32,
    pub beach_width: f32,
}

// Manual Default implementation matching assets/config/world_config.ron
impl Default for WorldEnvConfig {
    fn default() -> Self {
        Self {
            sea_level: 0.0,
            land_elevation: 3.0,
            spawn_drop_height: 10.0,
            ocean_floor_depth: -10.0,
            islands: IslandConfig::default(),
            terrain: TerrainConfig::default(),
            max_world_coordinate: 3000.0,
        }
    }
}

impl Default for IslandConfig {
    fn default() -> Self {
        Self {
            left_x: -1500.0,
            right_x: 1500.0,
            grid_x: 0.0,
            grid_z: 1800.0,
        }
    }
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            size: 1200.0,
            half_size: 600.0,
            beach_width: 100.0,
        }
    }
}

// All world environment constants have been migrated to WorldEnvConfig resource.
// Load from assets/config/world_config.ron at runtime.
// Access via: env: Res<WorldEnvConfig> or &config.world_env
