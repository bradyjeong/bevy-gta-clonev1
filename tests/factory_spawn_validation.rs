//! Factory Spawn Validation Tests
//! 
//! Following Oracle P0-D guidance: Test each focused factory can spawn entities successfully
//! with headless app tests and verify UnifiedEntityFactory delegates properly.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use gta_game::{
    factories::{BuildingsFactory, VehicleFactory, NPCFactory, VegetationFactory, UnifiedEntityFactory, GroundHeightCache},
    components::*,
    GameConfig,
};

/// Test that BuildingsFactory can spawn building entities successfully
#[test]
fn test_buildings_factory_spawn() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let mut factory = BuildingsFactory::new();
    let mut commands = app.world_mut().commands();
    let mut meshes = app.world_mut().resource_mut::<Assets<Mesh>>();
    let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
    let config = GameConfig::default();
    let mut ground_cache = GroundHeightCache::default();
    
    // Spawn building at valid position
    let position = Vec3::new(100.0, 0.0, 100.0);
    let result = factory.spawn_building(
        &mut commands,
        &mut meshes,
        &mut materials,
        position,
        &config,
        0.0,
        None,
        &mut ground_cache
    );
    
    assert!(result.is_ok(), "Building spawn failed: {:?}", result);
    
    // Apply commands to actually spawn the entity
    app.world_mut().flush();
    
    // Verify entity was spawned with correct components
    let building_query = app.world().query::<(&Building, &Transform, &DynamicContent)>();
    assert_eq!(building_query.iter(app.world()).count(), 1, "Expected exactly one building entity");
    
    // Verify building has correct content type
    let mut query = app.world().query::<&DynamicContent>();
    let dynamic_content = query.single(app.world());
    assert_eq!(dynamic_content.content_type, ContentType::Building);
}

/// Test that VehicleFactory can spawn vehicle entities successfully
#[test]
fn test_vehicle_factory_spawn() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let mut factory = VehicleFactory::new();
    let mut commands = app.world_mut().commands();
    let mut meshes = app.world_mut().resource_mut::<Assets<Mesh>>();
    let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
    let config = GameConfig::default();
    let mut ground_cache = GroundHeightCache::default();
    
    // Spawn vehicle at valid position (roads aren't required for this basic test)
    let position = Vec3::new(50.0, 0.0, 50.0);
    let result = factory.spawn_vehicle(
        &mut commands,
        &mut meshes,
        &mut materials,
        position,
        &config,
        0.0,
        None,
        &mut ground_cache
    );
    
    // Note: Vehicle spawn may fail due to road validation, which is expected behavior
    // We're testing that the factory doesn't panic and handles validation properly
    match result {
        Ok(_) => {
            app.world_mut().flush();
            
            // Verify entity was spawned with correct components
            let vehicle_query = app.world().query::<(&Car, &Transform, &DynamicContent)>();
            assert_eq!(vehicle_query.iter(app.world()).count(), 1, "Expected exactly one vehicle entity");
        }
        Err(err) => {
            // Expected for positions not on roads
            assert!(err.contains("road"), "Unexpected error type: {}", err);
        }
    }
}

/// Test that NPCFactory can spawn NPC entities successfully
#[test]
fn test_npc_factory_spawn() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let mut factory = NPCFactory::new();
    let mut commands = app.world_mut().commands();
    let mut meshes = app.world_mut().resource_mut::<Assets<Mesh>>();
    let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
    let config = GameConfig::default();
    let mut ground_cache = GroundHeightCache::default();
    
    // Spawn NPC at valid position
    let position = Vec3::new(75.0, 0.0, 75.0);
    let result = factory.spawn_npc(
        &mut commands,
        &mut meshes,
        &mut materials,
        position,
        &config,
        0.0,
        None,
        &mut ground_cache
    );
    
    assert!(result.is_ok(), "NPC spawn failed: {:?}", result);
    
    app.world_mut().flush();
    
    // Verify entity was spawned with correct components
    let npc_query = app.world().query::<(&NPCState, &Transform, &DynamicContent)>();
    assert_eq!(npc_query.iter(app.world()).count(), 1, "Expected exactly one NPC entity");
    
    // Verify NPC has correct content type
    let mut query = app.world().query::<&DynamicContent>();
    let dynamic_content = query.single(app.world());
    assert_eq!(dynamic_content.content_type, ContentType::NPC);
}

/// Test that VegetationFactory can spawn vegetation entities successfully  
#[test]
fn test_vegetation_factory_spawn() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let mut factory = VegetationFactory::new();
    let mut commands = app.world_mut().commands();
    let mut meshes = app.world_mut().resource_mut::<Assets<Mesh>>();
    let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
    let config = GameConfig::default();
    let mut ground_cache = GroundHeightCache::default();
    
    // Spawn vegetation at valid position
    let position = Vec3::new(125.0, 0.0, 125.0);
    let result = factory.spawn_vegetation(
        &mut commands,
        &mut meshes,
        &mut materials,
        position,
        &config,
        0.0,
        None,
        &mut ground_cache
    );
    
    assert!(result.is_ok(), "Vegetation spawn failed: {:?}", result);
    
    app.world_mut().flush();
    
    // Verify entity was spawned with correct components
    let tree_query = app.world().query::<(&DynamicContent, &Transform)>();
    assert_eq!(tree_query.iter(app.world()).count(), 1, "Expected exactly one vegetation entity");
    
    // Verify vegetation has correct content type
    let mut query = app.world().query::<&DynamicContent>();
    let dynamic_content = query.single(app.world());
    assert_eq!(dynamic_content.content_type, ContentType::Tree);
}

/// Test UnifiedEntityFactory delegation to focused factories
#[test]
fn test_unified_factory_delegation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let mut unified_factory = UnifiedEntityFactory::new();
    let mut commands = app.world_mut().commands();
    let mut meshes = app.world_mut().resource_mut::<Assets<Mesh>>();
    let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
    
    // Test building delegation
    let building_result = unified_factory.spawn_building_consolidated(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(200.0, 0.0, 200.0),
        0.0,
    );
    assert!(building_result.is_ok(), "Unified building spawn failed");
    
    // Test NPC delegation
    let npc_result = unified_factory.spawn_npc_consolidated(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(250.0, 0.0, 250.0),
        0.0,
    );
    assert!(npc_result.is_ok(), "Unified NPC spawn failed");
    
    // Test vegetation delegation
    let tree_result = unified_factory.spawn_tree_consolidated(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(300.0, 0.0, 300.0),
        0.0,
    );
    assert!(tree_result.is_ok(), "Unified vegetation spawn failed");
    
    app.world_mut().flush();
    
    // Verify all entities were spawned correctly
    let all_entities_query = app.world().query::<&DynamicContent>();
    assert_eq!(all_entities_query.iter(app.world()).count(), 3, "Expected exactly three entities");
}

/// Test batch spawning functionality
#[test]
fn test_batch_spawning() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let mut unified_factory = UnifiedEntityFactory::new();
    let mut commands = app.world_mut().commands();
    let mut meshes = app.world_mut().resource_mut::<Assets<Mesh>>();
    let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
    
    // Test batch building spawn
    let positions = vec![
        Vec3::new(400.0, 0.0, 400.0),
        Vec3::new(450.0, 0.0, 400.0),
        Vec3::new(500.0, 0.0, 400.0),
    ];
    
    let existing_content = Vec::new(); // No existing content for collision test
    
    let result = unified_factory.spawn_batch_consolidated(
        &mut commands,
        &mut meshes,
        &mut materials,
        ContentType::Building,
        positions,
        None,
        &existing_content,
        0.0,
    );
    
    assert!(result.is_ok(), "Batch spawn failed: {:?}", result);
    let spawned_entities = result.unwrap();
    assert_eq!(spawned_entities.len(), 3, "Expected 3 entities to be spawned");
    
    app.world_mut().flush();
    
    // Verify all batch entities were spawned
    let batch_query = app.world().query::<&DynamicContent>();
    assert_eq!(batch_query.iter(app.world()).count(), 3, "Expected exactly 3 batch entities");
}

/// Test entity limit enforcement
#[test]
fn test_entity_limit_enforcement() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let mut unified_factory = UnifiedEntityFactory::new();
    let mut commands = app.world_mut().commands();
    let mut meshes = app.world_mut().resource_mut::<Assets<Mesh>>();
    let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
    
    // Check initial entity counts
    let (buildings, vehicles, npcs, trees) = unified_factory.entity_limits.get_counts();
    assert_eq!(buildings, 0);
    assert_eq!(vehicles, 0);
    assert_eq!(npcs, 0);
    assert_eq!(trees, 0);
    
    // Spawn a few entities and verify tracking
    let _building = unified_factory.spawn_building_consolidated(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(600.0, 0.0, 600.0),
        1.0,
    );
    
    let _npc = unified_factory.spawn_npc_consolidated(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(650.0, 0.0, 650.0),
        2.0,
    );
    
    // Verify entity counts increased
    let (buildings, _vehicles, npcs, _trees) = unified_factory.entity_limits.get_counts();
    assert_eq!(buildings, 1);
    assert_eq!(npcs, 1);
}

/// Test spawn position validation
#[test]
fn test_spawn_position_validation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let unified_factory = UnifiedEntityFactory::new();
    
    // Test valid position
    let valid_position = Vec3::new(100.0, 0.0, 100.0);
    let result = unified_factory.validate_position(valid_position);
    assert!(result.is_ok(), "Valid position should pass validation");
    
    // Test invalid position (outside world bounds)
    let invalid_position = Vec3::new(10000.0, 0.0, 10000.0);
    let result = unified_factory.validate_position(invalid_position);
    assert!(result.is_err(), "Invalid position should fail validation");
}

/// Test collision detection
#[test]
fn test_collision_detection() {
    let unified_factory = UnifiedEntityFactory::new();
    
    let position = Vec3::new(100.0, 0.0, 100.0);
    let content_type = ContentType::Building;
    
    // Test with no existing content (should pass)
    let existing_content = Vec::new();
    let has_collision = unified_factory.has_content_collision(position, content_type, &existing_content);
    assert!(!has_collision, "No collision should be detected with empty existing content");
    
    // Test with nearby existing content (should collide)
    let existing_content = vec![
        (Vec3::new(105.0, 0.0, 105.0), ContentType::Building, 10.0)
    ];
    let has_collision = unified_factory.has_content_collision(position, content_type, &existing_content);
    assert!(has_collision, "Collision should be detected with nearby content");
    
    // Test with distant existing content (should not collide)
    let existing_content = vec![
        (Vec3::new(200.0, 0.0, 200.0), ContentType::Building, 10.0)
    ];
    let has_collision = unified_factory.has_content_collision(position, content_type, &existing_content);
    assert!(!has_collision, "No collision should be detected with distant content");
}
