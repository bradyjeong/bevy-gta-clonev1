use bevy::prelude::*;
use gta_game::systems::TerrainService;
use gta_game::systems::terrain::{TerrainConfig, WaterArea};

#[test]
fn test_terrain_service_with_config() {
    let mut terrain_service = TerrainService::default();
    
    // Initially should use fallback
    assert_eq!(terrain_service.height_at(0.0, 0.0), -0.15);
    assert!(!terrain_service.has_config());
    
    // Create a test configuration
    let config = TerrainConfig {
        world_size: 1000.0,
        resolution: 256,
        base_height: -0.5,
        hill_scale: 10.0,
        noise_seed: 54321,
        chunk_size: 32.0,
        water_areas: vec![
            WaterArea {
                center: (0.0, 0.0),
                radius: 25.0,
                depth: -2.0,
                description: "Test lake".to_string(),
            }
        ],
        ..Default::default()
    };
    
    // Update service with config
    terrain_service.update_config(config.clone());
    
    // Now should use config values
    assert!(terrain_service.has_config());
    assert_eq!(terrain_service.get_world_size(), 1000.0);
    assert_eq!(terrain_service.get_resolution(), 256);
    
    // Test height queries
    assert_eq!(terrain_service.height_at(100.0, 100.0), -0.5); // Land
    assert_eq!(terrain_service.height_at(0.0, 0.0), -2.0); // Water
    
    // Test water detection
    assert!(terrain_service.is_water_at(0.0, 0.0));
    assert!(!terrain_service.is_water_at(100.0, 100.0));
    
    // Test water depth
    assert_eq!(terrain_service.get_water_depth_at(0.0, 0.0), Some(-2.0));
    assert_eq!(terrain_service.get_water_depth_at(100.0, 100.0), None);
    
    // Test chunk calculations with config values
    let (chunk_x, chunk_z) = terrain_service.world_to_chunk(64.0, 64.0);
    assert_eq!(chunk_x, 2); // 64.0 / 32.0 = 2
    assert_eq!(chunk_z, 2);
    
    let world_pos = terrain_service.chunk_to_world(2, 2);
    assert_eq!(world_pos, Vec2::new(64.0, 64.0));
}

#[test]
fn test_terrain_config_validation() {
    let mut config = TerrainConfig::default();
    assert!(config.validate().is_ok());
    
    // Test invalid world size
    config.world_size = -100.0;
    assert!(config.validate().is_err());
    
    // Fix and test invalid resolution
    config.world_size = 1000.0;
    config.resolution = 10;
    assert!(config.validate().is_err());
    
    // Fix and test invalid chunk size
    config.resolution = 512;
    config.chunk_size = 0.0;
    assert!(config.validate().is_err());
}
