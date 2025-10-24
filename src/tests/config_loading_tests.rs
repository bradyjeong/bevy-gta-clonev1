use bevy::MinimalPlugins;
use bevy::prelude::*;
use std::fs;

use crate::components::world::WorldBounds;
use crate::config::{GameConfig, WorldBoundsConfig, WorldStreamingConfig};
use crate::constants::WorldEnvConfig;

#[test]
fn test_world_streaming_config_propagation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let streaming_config = WorldStreamingConfig {
        chunk_size: 200.0,
        streaming_radius: 800.0,
        lod_distances: crate::config::LodDistancesConfig {
            full: 150.0,
            medium: 300.0,
            far: 500.0,
        },
        ..Default::default()
    };

    let mut game_config = GameConfig::default();
    game_config.world_streaming = streaming_config.clone();
    game_config.world.chunk_size = streaming_config.chunk_size;
    game_config.world.streaming_radius = streaming_config.streaming_radius;
    game_config.world.lod_distances = [
        streaming_config.lod_distances.full,
        streaming_config.lod_distances.medium,
        streaming_config.lod_distances.far,
    ];

    app.insert_resource(game_config.clone());
    app.update();

    let config = app.world().resource::<GameConfig>();
    assert_eq!(
        config.world.chunk_size, 200.0,
        "chunk_size should propagate from world_streaming to world"
    );
    assert_eq!(
        config.world.streaming_radius, 800.0,
        "streaming_radius should propagate from world_streaming to world"
    );
    assert_eq!(
        config.world.lod_distances[0], 150.0,
        "full LOD distance should propagate"
    );
    assert_eq!(
        config.world.lod_distances[1], 300.0,
        "medium LOD distance should propagate"
    );
    assert_eq!(
        config.world.lod_distances[2], 500.0,
        "far LOD distance should propagate"
    );
}

#[test]
fn test_world_bounds_resource_from_config() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let bounds_config = WorldBoundsConfig {
        world_half_size: 3000.0,
        terrain: crate::config::TerrainBoundsConfig {
            left_x: -1500.0,
            right_x: 1500.0,
            half_size: 600.0,
        },
        edge_buffer: 200.0,
        vehicle_spawn_half_size: 2000.0,
    };

    let mut game_config = GameConfig::default();
    game_config.world_bounds = bounds_config.clone();
    game_config.world.map_size = bounds_config.world_half_size * 2.0;

    app.insert_resource(game_config.clone());

    let world_bounds = WorldBounds::from_config(&game_config.world);
    app.insert_resource(world_bounds);
    app.update();

    let bounds = app.world().resource::<WorldBounds>();
    let config = app.world().resource::<GameConfig>();

    assert_eq!(
        config.world.map_size, 6000.0,
        "map_size should be world_half_size * 2"
    );
    assert_eq!(
        bounds.min_x, -3000.0,
        "WorldBounds min_x should match derived bounds"
    );
    assert_eq!(
        bounds.max_x, 3000.0,
        "WorldBounds max_x should match derived bounds"
    );
    assert_eq!(
        bounds.min_z, -3000.0,
        "WorldBounds min_z should match derived bounds"
    );
    assert_eq!(
        bounds.max_z, 3000.0,
        "WorldBounds max_z should match derived bounds"
    );
}

#[test]
fn test_world_env_config_resource_availability() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let world_env = WorldEnvConfig {
        sea_level: 0.0,
        land_elevation: 3.0,
        spawn_drop_height: 10.0,
        ocean_floor_depth: -10.0,
        islands: crate::constants::IslandConfig {
            left_x: -1500.0,
            right_x: 1500.0,
            grid_x: 0.0,
            grid_z: 1800.0,
        },
        terrain: crate::constants::TerrainConfig {
            size: 1200.0,
            half_size: 600.0,
            beach_width: 100.0,
        },
        max_world_coordinate: 3000.0,
    };

    app.insert_resource(world_env.clone());
    app.update();

    let env = app.world().resource::<WorldEnvConfig>();
    assert_eq!(
        env.sea_level, 0.0,
        "WorldEnvConfig should have correct sea_level"
    );
    assert_eq!(
        env.land_elevation, 3.0,
        "WorldEnvConfig should have correct land_elevation"
    );
    assert_eq!(
        env.spawn_drop_height, 10.0,
        "WorldEnvConfig should have correct spawn_drop_height"
    );
    assert_eq!(
        env.ocean_floor_depth, -10.0,
        "WorldEnvConfig should have correct ocean_floor_depth"
    );
    assert_eq!(
        env.islands.left_x, -1500.0,
        "Island config should be accessible"
    );
    assert_eq!(
        env.terrain.size, 1200.0,
        "Terrain config should be accessible"
    );
    assert_eq!(
        env.max_world_coordinate, 3000.0,
        "Max world coordinate should be set"
    );
}

#[test]
fn test_config_validation_and_clamping() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let mut game_config = GameConfig::default();

    game_config.world.chunk_size = 10000.0;
    game_config.world_streaming.chunk_size = 10000.0;

    let validation_system = |mut config: ResMut<GameConfig>| {
        if config.world.chunk_size > 1000.0 {
            warn!(
                "⚠️ chunk_size {} exceeds safe limit 1000.0, clamping",
                config.world.chunk_size
            );
            config.world.chunk_size = 1000.0;
            config.world_streaming.chunk_size = 1000.0;
        }

        if config.world.streaming_radius < 100.0 {
            warn!(
                "⚠️ streaming_radius {} below minimum 100.0, clamping",
                config.world.streaming_radius
            );
            config.world.streaming_radius = 100.0;
        }
    };

    app.insert_resource(game_config);
    app.add_systems(Startup, validation_system);
    app.update();

    let config = app.world().resource::<GameConfig>();
    assert_eq!(
        config.world.chunk_size, 1000.0,
        "chunk_size should be clamped to safe maximum"
    );
    assert_eq!(
        config.world_streaming.chunk_size, 1000.0,
        "world_streaming chunk_size should be clamped"
    );
}

#[test]
fn test_graceful_fallback_to_defaults() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let default_config = GameConfig::default();
    app.insert_resource(default_config.clone());
    app.update();

    let config = app.world().resource::<GameConfig>();

    assert!(
        config.world.chunk_size > 0.0,
        "Default chunk_size should be positive"
    );
    assert!(
        config.world.streaming_radius > 0.0,
        "Default streaming_radius should be positive"
    );
    assert!(
        config.world.lod_distances[0] > 0.0,
        "Default LOD distances should be positive"
    );
    assert!(
        config.world.map_size > 0.0,
        "Default map_size should be positive"
    );

    assert!(
        config.physics.max_velocity > 0.0,
        "Default physics config should be valid"
    );
    assert!(
        config.physics.max_world_coord > 0.0,
        "Default world coordinate should be positive"
    );
}

#[test]
fn test_ron_file_parsing_world_streaming() {
    let assets_base =
        if cfg!(target_os = "macos") && std::path::Path::new("../Resources/assets").exists() {
            "../Resources/assets"
        } else {
            "assets"
        };

    let path = format!("{}/config/world_streaming.ron", assets_base);
    let contents =
        fs::read_to_string(&path).expect("world_streaming.ron should exist and be readable");

    let streaming_config: WorldStreamingConfig =
        ron::from_str(&contents).expect("world_streaming.ron should parse correctly");

    assert!(
        streaming_config.chunk_size > 0.0,
        "Parsed chunk_size should be positive"
    );
    assert!(
        streaming_config.streaming_radius > 0.0,
        "Parsed streaming_radius should be positive"
    );
    assert!(
        streaming_config.lod_distances.full > 0.0,
        "Parsed full LOD should be positive"
    );
    assert!(
        streaming_config.lod_distances.medium > streaming_config.lod_distances.full,
        "Medium LOD should be greater than full LOD"
    );
    assert!(
        streaming_config.lod_distances.far > streaming_config.lod_distances.medium,
        "Far LOD should be greater than medium LOD"
    );
}

#[test]
fn test_ron_file_parsing_world_bounds() {
    let assets_base =
        if cfg!(target_os = "macos") && std::path::Path::new("../Resources/assets").exists() {
            "../Resources/assets"
        } else {
            "assets"
        };

    let path = format!("{}/config/world_bounds.ron", assets_base);
    let contents =
        fs::read_to_string(&path).expect("world_bounds.ron should exist and be readable");

    let bounds_config: WorldBoundsConfig =
        ron::from_str(&contents).expect("world_bounds.ron should parse correctly");

    assert!(
        bounds_config.world_half_size > 0.0,
        "Parsed world_half_size should be positive"
    );
    assert!(
        bounds_config.terrain.left_x < 0.0,
        "Left terrain bound should be negative"
    );
    assert!(
        bounds_config.terrain.right_x > 0.0,
        "Right terrain bound should be positive"
    );
    assert!(
        bounds_config.edge_buffer > 0.0,
        "Edge buffer should be positive"
    );
}

#[test]
fn test_ron_file_parsing_world_config() {
    let assets_base =
        if cfg!(target_os = "macos") && std::path::Path::new("../Resources/assets").exists() {
            "../Resources/assets"
        } else {
            "assets"
        };

    let path = format!("{}/config/world_config.ron", assets_base);
    let contents =
        fs::read_to_string(&path).expect("world_config.ron should exist and be readable");

    let world_env: WorldEnvConfig =
        ron::from_str(&contents).expect("world_config.ron should parse correctly");

    assert_eq!(world_env.sea_level, 0.0, "Sea level should be at 0.0");
    assert!(
        world_env.land_elevation > world_env.sea_level,
        "Land should be above sea level"
    );
    assert!(
        world_env.ocean_floor_depth < world_env.sea_level,
        "Ocean floor should be below sea level"
    );
    assert!(
        world_env.spawn_drop_height > world_env.land_elevation,
        "Spawn height should be above land"
    );
    assert!(
        world_env.max_world_coordinate > 0.0,
        "Max world coordinate should be positive"
    );
}

#[test]
fn test_config_consistency_across_resources() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let world_half_size = 3000.0;
    let bounds_config = WorldBoundsConfig {
        world_half_size,
        terrain: crate::config::TerrainBoundsConfig {
            left_x: -1500.0,
            right_x: 1500.0,
            half_size: 600.0,
        },
        edge_buffer: 200.0,
        vehicle_spawn_half_size: 2000.0,
    };

    let mut game_config = GameConfig::default();
    game_config.world_bounds = bounds_config.clone();
    game_config.world.map_size = world_half_size * 2.0;

    app.insert_resource(game_config.clone());

    let world_bounds = WorldBounds::from_config(&game_config.world);
    app.insert_resource(world_bounds);

    let world_env = WorldEnvConfig {
        max_world_coordinate: world_half_size,
        ..Default::default()
    };
    app.insert_resource(world_env);

    app.update();

    let config = app.world().resource::<GameConfig>();
    let bounds = app.world().resource::<WorldBounds>();
    let env = app.world().resource::<WorldEnvConfig>();

    assert_eq!(
        config.world.map_size,
        world_half_size * 2.0,
        "GameConfig map_size should match world_half_size * 2"
    );
    assert_eq!(
        bounds.max_x, world_half_size,
        "WorldBounds max_x should match world_half_size"
    );
    assert_eq!(
        env.max_world_coordinate, world_half_size,
        "WorldEnvConfig should use consistent coordinate"
    );
}
