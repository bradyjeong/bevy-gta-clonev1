use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use serde::Deserialize;

// Collision groups for proper physics separation (implementation constant - not data-driven)
pub const STATIC_GROUP: Group = Group::GROUP_1; // Buildings, terrain, trees
pub const VEHICLE_GROUP: Group = Group::GROUP_2; // Cars, helicopters, jets
pub const CHARACTER_GROUP: Group = Group::GROUP_3; // Player, NPCs

// World environment configuration loaded from assets/config/world_config.ron
// Renamed to WorldEnvConfig to avoid collision with config::WorldConfig
#[derive(Debug, Clone, Resource, Deserialize, Default)]
pub struct WorldEnvConfig {
    pub sea_level: f32,
    pub land_elevation: f32,
    pub spawn_drop_height: f32,
    pub ocean_floor_depth: f32,
    pub islands: IslandConfig,
    pub terrain: TerrainConfig,
    pub max_world_coordinate: f32,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct IslandConfig {
    pub left_x: f32,
    pub right_x: f32,
    pub grid_x: f32,
    pub grid_z: f32,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TerrainConfig {
    pub size: f32,
    pub half_size: f32,
    pub beach_width: f32,
}

// Legacy constants for backwards compatibility - DEPRECATED, use WorldEnvConfig resource instead
#[deprecated(since = "0.2.0", note = "Use WorldEnvConfig resource: env.sea_level")]
pub const SEA_LEVEL: f32 = 0.0;
#[deprecated(
    since = "0.2.0",
    note = "Use WorldEnvConfig resource: env.land_elevation"
)]
pub const LAND_ELEVATION: f32 = 3.0;
#[deprecated(
    since = "0.2.0",
    note = "Use WorldEnvConfig resource: env.spawn_drop_height"
)]
pub const SPAWN_DROP_HEIGHT: f32 = 10.0;
#[deprecated(
    since = "0.2.0",
    note = "Use WorldEnvConfig resource: env.ocean_floor_depth"
)]
pub const OCEAN_FLOOR_DEPTH: f32 = -10.0;
#[deprecated(
    since = "0.2.0",
    note = "Use WorldEnvConfig resource: env.islands.left_x"
)]
pub const LEFT_ISLAND_X: f32 = -1500.0;
#[deprecated(
    since = "0.2.0",
    note = "Use WorldEnvConfig resource: env.islands.right_x"
)]
pub const RIGHT_ISLAND_X: f32 = 1500.0;
#[deprecated(
    since = "0.2.0",
    note = "Use WorldEnvConfig resource: env.islands.grid_x"
)]
pub const GRID_ISLAND_X: f32 = 0.0;
#[deprecated(
    since = "0.2.0",
    note = "Use WorldEnvConfig resource: env.islands.grid_z"
)]
pub const GRID_ISLAND_Z: f32 = 1800.0;
#[deprecated(
    since = "0.2.0",
    note = "Use WorldEnvConfig resource: env.terrain.size"
)]
pub const TERRAIN_SIZE: f32 = 1200.0;
#[deprecated(
    since = "0.2.0",
    note = "Use WorldEnvConfig resource: env.terrain.half_size"
)]
pub const TERRAIN_HALF_SIZE: f32 = 600.0;
#[deprecated(
    since = "0.2.0",
    note = "Use WorldEnvConfig resource: env.terrain.beach_width"
)]
pub const BEACH_WIDTH: f32 = 100.0;
#[deprecated(
    since = "0.2.0",
    note = "Use WorldEnvConfig resource: env.max_world_coordinate"
)]
pub const MAX_WORLD_COORDINATE: f32 = 3000.0;
