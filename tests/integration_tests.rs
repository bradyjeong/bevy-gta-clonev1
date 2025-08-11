use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use gta_game::components::*;
use gta_game::game_state::GameState;

use gta_game::systems::input::InputManager;
use gta_game::factories::entity_factory_unified::UnifiedEntityFactory;
use gta_game::systems::world::unified_distance_culling::*;
use gta_game::services::distance_cache::DistanceCache;

/// Integration test for core game components and systems
#[test]
fn test_core_game_initialization() {
    let mut app = App::new();
    
    // Add minimal plugins for core functionality
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .init_state::<GameState>()
        .insert_state(GameState::Driving);
    
    // Add just the resources we want to test
    app.insert_resource(InputManager::default());
    
    // Run a single update to ensure everything initializes
    app.update();
    
    // Verify game state resource exists
    assert!(app.world().contains_resource::<State<GameState>>());
    assert!(app.world().contains_resource::<InputManager>());
}

/// Test SuperCar component system integration
#[test]
fn test_supercar_component_system() {
    let mut app = App::new();
    
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .init_asset::<Mesh>()
        .init_state::<GameState>()
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
    
    // Create a SuperCar entity with new component system
    let supercar_entity = app.world_mut().spawn((
        Car,
        SuperCarBundle::default(),
        Transform::default(),
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 0.5, 2.0),
        Velocity::default(),
        Name::new("TestSuperCar"),
    )).id();
    
    app.update();
    
    // Verify all components exist
    let world = app.world();
    assert!(world.entity(supercar_entity).contains::<Car>());
    assert!(world.entity(supercar_entity).contains::<SuperCarSpecs>());
    assert!(world.entity(supercar_entity).contains::<EngineState>());
    assert!(world.entity(supercar_entity).contains::<TurboSystem>());
    assert!(world.entity(supercar_entity).contains::<DrivingModes>());
    assert!(world.entity(supercar_entity).contains::<PerformanceMetrics>());
    
    // Test component values
    let specs = world.entity(supercar_entity).get::<SuperCarSpecs>().unwrap();
    assert!(specs.max_speed > 0.0);
    assert!(specs.power > 0.0);
    
    let engine = world.entity(supercar_entity).get::<EngineState>().unwrap();
    assert!(engine.max_rpm > engine.idle_rpm);
}

/// Test distance culling system with entities
#[test]
fn test_distance_culling_integration() {
    let mut app = App::new();
    
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .init_asset::<Mesh>()
        .init_state::<GameState>()
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_systems(Update, (
            new_unified_distance_culling_system,
            unified_culling_movement_tracker,
        ))
        .insert_resource(DistanceCache::new())
        .insert_resource(UnifiedCullingTimer::default())
        .init_resource::<FrameCounter>();
    
    // Create active entity (player)
    let _player = app.world_mut().spawn((
        ActiveEntity,
        Transform::from_translation(Vec3::ZERO),
        Name::new("Player"),
    )).id();
    
    // Create cullable entities at various distances
    let near_entity = app.world_mut().spawn((
        UnifiedCullable::building(),
        Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)),
        Name::new("NearBuilding"),
    )).id();
    
    let far_entity = app.world_mut().spawn((
        UnifiedCullable::building(),
        Transform::from_translation(Vec3::new(500.0, 0.0, 0.0)),
        Name::new("FarBuilding"),
    )).id();
    
    // Run several updates to process culling
    for _ in 0..5 {
        app.update();
    }
    
    // Verify entities still exist (culling system should mark them, not delete them)
    assert!(app.world().get_entity(near_entity).is_ok());
    assert!(app.world().get_entity(far_entity).is_ok());
    
    // Check that culling data was updated
    let near_cullable = app.world().entity(near_entity).get::<UnifiedCullable>().unwrap();
    let far_cullable = app.world().entity(far_entity).get::<UnifiedCullable>().unwrap();
    
    // Near entity should have smaller distance
    assert!(near_cullable.last_distance < far_cullable.last_distance);
}

/// Test vehicle factory integration
#[test]
fn test_vehicle_factory_integration() {
    let mut app = App::new();
    
    app.add_plugins(MinimalPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(UnifiedEntityFactory::new());
    
    // Get the factory
    let factory = app.world().resource::<UnifiedEntityFactory>();
    
    // Test factory configuration
    assert!(factory.config.vehicles.basic_car.max_speed > 0.0);
    assert!(factory.config.vehicles.super_car.max_speed > factory.config.vehicles.basic_car.max_speed);
    
    // Test entity limits exist
    assert!(factory.entity_limits.max_buildings > 0);
    assert!(factory.entity_limits.max_vehicles > 0);
}

/*
/// Test control manager integration
/// DISABLED: ControlManager removed in favor of asset-based controls
#[test]
fn test_control_manager_integration() {
    // This test is disabled because ControlManager was removed
    // in favor of the new asset-based control system
}
*/

/// Test component bundle consistency
#[test]
fn test_component_bundle_consistency() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Test SuperCarBundle default values
    let supercar_bundle = SuperCarBundle::default();
    
    // Verify specs are reasonable
    assert!(supercar_bundle.specs.max_speed > 100.0); // Should be fast
    assert!(supercar_bundle.specs.acceleration > 0.0);
    assert!(supercar_bundle.specs.power > 0.0);
    
    // Verify engine state
    assert!(supercar_bundle.engine.max_rpm > supercar_bundle.engine.idle_rpm);
    assert!(supercar_bundle.engine.power_band_start < supercar_bundle.engine.power_band_end);
    
    // Verify turbo system
    assert!(supercar_bundle.turbo.max_time > 0.0);
    assert!(supercar_bundle.turbo.stage == 0); // Should start at stage 0
    
    // Verify driving modes - updated to match default value
    assert!(matches!(supercar_bundle.driving_modes.mode, DrivingMode::Sport));
    assert!(!supercar_bundle.driving_modes.launch_control_engaged); // Should start disengaged
}

/// Test system integration with realistic scenarios
#[test]
fn test_realistic_scenario_integration() {
    let mut app = App::new();
    
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .init_asset::<Mesh>()
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .init_state::<GameState>()
        .insert_resource(DistanceCache::new())
        .insert_resource(UnifiedCullingTimer::default())
        .insert_resource(InputManager::default())

        .insert_state(GameState::Driving);
    
    // Create a realistic game scenario
    let player = app.world_mut().spawn((
        Player,
        ActiveEntity,
        Transform::from_translation(Vec3::new(10.0, 1.0, 10.0)),
        RigidBody::Dynamic,
        Collider::capsule_y(0.5, 0.5),
        Velocity::default(),
        Name::new("Player"),
    )).id();
    
    let supercar = app.world_mut().spawn((
        Car,
        SuperCarBundle::default(),
        Transform::from_translation(Vec3::new(15.0, 0.0, 10.0)),
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 0.5, 2.0),
        Velocity::default(),
        Name::new("PlayerSuperCar"),
    )).id();
    
    let building = app.world_mut().spawn((
        UnifiedCullable::building(),
        Transform::from_translation(Vec3::new(100.0, 0.0, 100.0)),
        Name::new("DistantBuilding"),
    )).id();
    
    // Run simulation for several frames
    for _ in 0..10 {
        app.update();
    }
    
    // Verify all entities still exist and have expected components
    let world = app.world();
    
    assert!(world.get_entity(player).is_ok());
    assert!(world.entity(player).contains::<Player>());
    assert!(world.entity(player).contains::<ActiveEntity>());
    
    assert!(world.get_entity(supercar).is_ok());
    assert!(world.entity(supercar).contains::<Car>());
    assert!(world.entity(supercar).contains::<SuperCarSpecs>());
    
    assert!(world.get_entity(building).is_ok());
    assert!(world.entity(building).contains::<UnifiedCullable>());
}

/// Test error handling and edge cases
#[test]
fn test_error_handling() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Test with invalid entity
    let invalid_entity = Entity::from_raw(99999);
    assert!(app.world().get_entity(invalid_entity).is_err());
    
    // Test distance cache with no entries
    let distance_cache = DistanceCache::new();
    assert_eq!(distance_cache.len(), 0);
    
    // Note: ControlManager removed in favor of asset-based controls
}

/// Performance test to ensure systems don't regress
#[test]
fn test_system_performance() {
    let mut app = App::new();
    
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .init_asset::<Mesh>()
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(DistanceCache::new())
        .insert_resource(UnifiedCullingTimer::default())
        .init_resource::<FrameCounter>()
        .add_systems(Update, new_unified_distance_culling_system);
    
    // Create player
    let _player = app.world_mut().spawn((
        ActiveEntity,
        Transform::from_translation(Vec3::ZERO),
    )).id();
    
    // Create many cullable entities
    for i in 0..100 {
        app.world_mut().spawn((
            UnifiedCullable::building(),
            Transform::from_translation(Vec3::new(i as f32 * 10.0, 0.0, 0.0)),
        ));
    }
    
    // Measure time for updates
    let start = std::time::Instant::now();
    
    for _ in 0..10 {
        app.update();
    }
    
    let duration = start.elapsed();
    
    // Should complete reasonably quickly (adjust threshold as needed)
    assert!(duration.as_millis() < 100, "System performance regression detected: {}ms", duration.as_millis());
}
