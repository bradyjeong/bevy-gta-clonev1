use bevy::prelude::*;
use gta_game::config::{GameConfig, ConfigReloadedEvent, ConfigPlugin};
use gta_game::config::gameplay::GameplayConfig;
use gta_game::config::performance::PerformanceConfig;
use gta_game::config::debug::DebugConfig;

#[test]
fn test_config_loading() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(ConfigPlugin);

    app.update();

    // Verify config resource exists
    let config = app.world().resource::<GameConfig>();
    assert!(config.performance.culling.building_distance == 300.0);
    assert!(config.performance.culling.vehicle_distance == 150.0);
    assert!(config.performance.culling.npc_distance == 100.0);
}

#[test]
fn test_config_reload_event() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(ConfigPlugin)
        .add_event::<ConfigReloadedEvent>();

    // Modify config
    let mut config = app.world_mut().resource_mut::<GameConfig>();
    config.performance.culling.building_distance = 400.0;

    app.update();

    // Check if reload event was sent
    let events = app.world().resource::<Events<ConfigReloadedEvent>>();
    assert!(events.len() > 0);
}

#[test]
fn test_performance_config_values() {
    let config = PerformanceConfig::default();
    
    // Test culling distances
    assert_eq!(config.culling.building_distance, 300.0);
    assert_eq!(config.culling.vehicle_distance, 150.0);
    assert_eq!(config.culling.npc_distance, 100.0);
    
    // Test cache sizes
    assert_eq!(config.caching.distance_cache_size, 2048);
    assert_eq!(config.caching.distance_cache_ttl, 5);
    
    // Test entity limits
    assert_eq!(config.entity_limits.max_buildings, 200);
    assert_eq!(config.entity_limits.max_vehicles, 50);
    assert_eq!(config.entity_limits.max_npcs, 20);
    
    // Test timing intervals
    assert_eq!(config.timing.culling_update_interval, 0.5);
    assert_eq!(config.timing.lod_update_interval, 0.2);
}

#[test]
fn test_gameplay_config_values() {
    let config = GameplayConfig::default();
    
    // Test vehicle speeds
    assert_eq!(config.vehicle.car_top_speed, 120.0);
    assert_eq!(config.vehicle.supercar_top_speed, 200.0);
    assert_eq!(config.vehicle.f16_max_speed, 600.0);
    
    // Test NPC values
    assert_eq!(config.npc.walk_speed, 2.0);
    assert_eq!(config.npc.run_speed, 5.0);
    assert_eq!(config.npc.vision_range, 50.0);
    
    // Test physics values
    assert_eq!(config.physics.friction_coefficient, 0.6);
    assert_eq!(config.physics.air_resistance, 0.1);
}

#[test]
fn test_debug_config_defaults() {
    let config = DebugConfig::default();
    
    // Test overlay defaults
    assert!(!config.overlays.show_fps);
    assert!(!config.overlays.show_physics_debug);
    assert_eq!(config.overlays.overlay_opacity, 0.8);
    
    // Test logging defaults
    assert_eq!(config.logging.level, "info");
    assert!(!config.logging.log_events);
    assert_eq!(config.logging.performance_threshold_ms, 16.0);
    
    // Test cheat defaults
    assert!(!config.cheats.god_mode);
    assert!(!config.cheats.infinite_fuel);
}

#[test]
fn test_config_serialization() {
    let config = GameConfig::default();
    
    // Serialize to RON
    let serialized = ron::to_string(&config).expect("Failed to serialize config");
    assert!(serialized.contains("culling"));
    assert!(serialized.contains("building_distance"));
    
    // Deserialize back
    let deserialized: GameConfig = ron::from_str(&serialized).expect("Failed to deserialize config");
    assert_eq!(deserialized.performance.culling.building_distance, config.performance.culling.building_distance);
}

/// Test that configuration can be modified at runtime
#[test]
fn test_runtime_config_modification() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(ConfigPlugin);

    app.update();

    // Modify configuration
    {
        let mut config = app.world_mut().resource_mut::<GameConfig>();
        config.performance.culling.building_distance = 500.0;
        config.gameplay.vehicle.car_top_speed = 150.0;
    }

    app.update();

    // Verify changes persisted
    let config = app.world().resource::<GameConfig>();
    assert_eq!(config.performance.culling.building_distance, 500.0);
    assert_eq!(config.gameplay.vehicle.car_top_speed, 150.0);
}

/// Test cross-plugin configuration access
#[test]
fn test_cross_plugin_config_access() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(ConfigPlugin)
        .add_systems(Update, test_config_reader);

    app.update();
}

fn test_config_reader(config: Res<GameConfig>) {
    // Systems can read configuration values
    assert!(config.performance.culling.building_distance > 0.0);
    assert!(config.performance.entity_limits.max_vehicles > 0);
    assert!(config.gameplay.physics.gravity > 0.0);
}

/// Performance test for configuration access
#[test]
fn test_config_access_performance() {
    use std::time::Instant;
    
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(ConfigPlugin);

    app.update();

    let start = Instant::now();
    
    // Access configuration many times
    for _ in 0..10000 {
        let config = app.world().resource::<GameConfig>();
        let _ = config.performance.culling.building_distance;
        let _ = config.gameplay.vehicle.car_top_speed;
        let _ = config.debug.overlays.show_fps;
    }
    
    let duration = start.elapsed();
    
    // Should complete in under 10ms
    assert!(duration.as_millis() < 10, "Config access took {}ms", duration.as_millis());
}
