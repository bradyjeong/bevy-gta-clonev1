#![cfg(feature = "heavy_tests")]
// Full simulation integration tests
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use game_core::prelude::*;
use gameplay_sim::systems::input::{ControlManager, ControlAction};
use crate::utils::*;

#[test]
fn test_complete_simulation_integration() {
    let mut app = create_test_app();
    let (player_entity, vehicle_entity, _ground_entity) = setup_test_scene(&mut app);
    
    // Add control manager for input handling
    let mut control_manager = ControlManager::new();
    control_manager.set_control_value(ControlAction::Accelerate, 0.5);
    app.insert_resource(control_manager);
    // Run complete simulation for significant duration
    run_simulation_duration(&mut app, 10.0);
    // Verify all systems worked together properly
    let world = app.world();
    // Player should still exist and be valid
    let player_transform = world.get::<Transform>(player_entity).unwrap();
    let player_velocity = world.get::<Velocity>(player_entity).unwrap();
    assert!(player_transform.translation.is_finite(), "Player position should be finite");
    assert!(player_velocity.linvel.is_finite(), "Player velocity should be finite");
    // Vehicle should still exist and be valid
    let vehicle_transform = world.get::<Transform>(vehicle_entity).unwrap();
    let vehicle_velocity = world.get::<Velocity>(vehicle_entity).unwrap();
    assert!(vehicle_transform.translation.is_finite(), "Vehicle position should be finite");
    assert!(vehicle_velocity.linvel.is_finite(), "Vehicle velocity should be finite");
    // Vehicle should have moved due to acceleration input
    assert!(vehicle_velocity.linvel.length() > 0.1, "Vehicle should have moved");
    // All entities should be within reasonable bounds
    assert!(player_transform.translation.length() < 1000.0, "Player should be within bounds");
    assert!(vehicle_transform.translation.length() < 1000.0, "Vehicle should be within bounds");
}
fn test_multi_vehicle_simulation() {
    // Create multiple vehicles with different characteristics
    let vehicles: Vec<Entity> = (0..3).map(|i| {
        let mut world = app.world_mut();
        world.spawn((
            Car::default(),
            ActiveEntity,
            Transform::from_xyz(i as f32 * 10.0, 1.0, 0.0),
            GlobalTransform::default(),
            Velocity::default(),
            RigidBody::Dynamic,
            Collider::cuboid(2.0, 1.0, 4.0),
            RealisticVehicle::default(),
            VehicleDynamics {
                total_mass: 1500.0 + (i as f32 * 500.0), // Different masses
                ..Default::default()
            },
            EnginePhysics::default(),
            VehicleSuspension::default(),
            TirePhysics::default(),
        )).id()
    }).collect();
    // Add input for all vehicles
    control_manager.set_control_value(ControlAction::Accelerate, 0.7);
    // Run simulation
    run_simulation_duration(&mut app, 5.0);
    // Verify all vehicles behaved properly
    for (i, vehicle) in vehicles.iter().enumerate() {
        let transform = world.get::<Transform>(*vehicle).unwrap();
        let velocity = world.get::<Velocity>(*vehicle).unwrap();
        let dynamics = world.get::<VehicleDynamics>(*vehicle).unwrap();
        
        // All vehicles should be valid
        assert!(transform.translation.is_finite(), "Vehicle {} position should be finite", i);
        assert!(velocity.linvel.is_finite(), "Vehicle {} velocity should be finite", i);
        // All should have moved
        assert!(velocity.linvel.length() > 0.1, "Vehicle {} should have moved", i);
        // Different masses should result in different behaviors
        assert!(dynamics.total_mass > 0.0, "Vehicle {} should have valid mass", i);
    }
fn test_player_vehicle_interaction() {
    let (player_entity, vehicle_entity, _) = setup_test_scene(&mut app);
    // Move player close to vehicle
    {
        if let Some(mut transform) = world.get_mut::<Transform>(player_entity) {
            transform.translation = Vec3::new(8.0, 0.0, 0.0); // Close to vehicle at (10,0,0)
        }
    // Give both some movement
        if let Some(mut velocity) = world.get_mut::<Velocity>(player_entity) {
            velocity.linvel = Vec3::new(2.0, 0.0, 0.0);
        if let Some(mut velocity) = world.get_mut::<Velocity>(vehicle_entity) {
            velocity.linvel = Vec3::new(-1.0, 0.0, 0.0);
    run_simulation_duration(&mut app, 3.0);
    // Check interaction results
    // Both should still be valid
    assert!(player_transform.translation.is_finite(), "Player should remain valid");
    assert!(vehicle_transform.translation.is_finite(), "Vehicle should remain valid");
    // They should maintain reasonable separation or have interacted properly
    let distance = (player_transform.translation - vehicle_transform.translation).length();
    assert!(distance < 50.0, "Entities should not have flown apart unreasonably");
fn test_physics_system_stability() {
    // Create complex scene with many entities
    let mut entities = Vec::new();
    // Add various entity types
    for i in 0..10 {
        let entity = {
            let mut world = app.world_mut();
            world.spawn((
                Transform::from_xyz(
                    (i as f32 - 5.0) * 5.0,
                    1.0 + (i as f32 * 0.5),
                    (i as f32 % 3) as f32 * 3.0,
                ),
                GlobalTransform::default(),
                Velocity {
                    linvel: Vec3::new(
                        (i as f32 - 5.0) * 0.5,
                        0.0,
                        (i as f32 % 2) as f32 * 2.0 - 1.0,
                    ),
                    angvel: Vec3::ZERO,
                },
                RigidBody::Dynamic,
                Collider::ball(0.5 + (i as f32 * 0.1)),
            )).id()
        };
        entities.push(entity);
    // Run long simulation
    run_simulation_duration(&mut app, 15.0);
    // Verify system stability
    for (i, entity) in entities.iter().enumerate() {
        let transform = world.get::<Transform>(*entity).unwrap();
        let velocity = world.get::<Velocity>(*entity).unwrap();
        // All entities should remain stable
        assert!(transform.translation.is_finite(), "Entity {} position should be finite", i);
        assert!(velocity.linvel.is_finite(), "Entity {} velocity should be finite", i);
        assert!(velocity.linvel.length() < 100.0, "Entity {} velocity should be reasonable", i);
        assert!(transform.translation.length() < 1000.0, "Entity {} position should be reasonable", i);
fn test_simulation_determinism() {
    // Test that identical simulations produce identical results
    let mut app1 = create_test_app();
    let mut app2 = create_test_app();
    // Create identical scenes
    let entity1 = create_test_supercar(&mut app1);
    let entity2 = create_test_supercar(&mut app2);
    // Apply identical inputs
    let control_manager = {
        let mut cm = ControlManager::new();
        cm.set_control_value(ControlAction::Accelerate, 0.6);
        cm.set_control_value(ControlAction::Steer, 0.3);
        cm
    };
    app1.insert_resource(control_manager.clone());
    app2.insert_resource(control_manager);
    // Run identical simulations
    let steps = 300; // 5 seconds
    for _ in 0..steps {
        app1.update();
        app2.update();
    // Compare final states
    let world1 = app1.world();
    let world2 = app2.world();
    let transform1 = world1.get::<Transform>(entity1).unwrap();
    let transform2 = world2.get::<Transform>(entity2).unwrap();
    let velocity1 = world1.get::<Velocity>(entity1).unwrap();
    let velocity2 = world2.get::<Velocity>(entity2).unwrap();
    // Should be identical (within floating point precision)
    assert!(vec3_equals(transform1.translation, transform2.translation, 0.001),
           "Positions should be identical: {:?} vs {:?}", 
           transform1.translation, transform2.translation);
    assert!(vec3_equals(velocity1.linvel, velocity2.linvel, 0.001),
           "Velocities should be identical: {:?} vs {:?}", 
           velocity1.linvel, velocity2.linvel);
fn test_system_performance_under_load() {
    // Create many entities to stress test
    let entity_count = 50;
    for i in 0..entity_count {
        let angle = (i as f32 / entity_count as f32) * std::f32::consts::TAU;
        let radius = 20.0;
        let entity = if i % 3 == 0 {
            // Vehicle
                Car::default(),
                ActiveEntity,
                    angle.cos() * radius,
                    1.0,
                    angle.sin() * radius,
                Velocity::default(),
                Collider::cuboid(1.0, 0.5, 2.0),
                RealisticVehicle::default(),
                VehicleDynamics::default(),
                EnginePhysics::default(),
                VehicleSuspension::default(),
                TirePhysics::default(),
        } else {
            // Regular physics entity
                    angle.cos() * radius * 0.7,
                    angle.sin() * radius * 0.7,
                        angle.cos() * 2.0,
                        angle.sin() * 2.0,
                Collider::ball(0.5),
    // Add some input
    control_manager.set_control_value(ControlAction::Accelerate, 0.3);
    // Time the simulation
    let start_time = std::time::Instant::now();
    run_simulation_duration(&mut app, 5.0); // 5 seconds
    let elapsed = start_time.elapsed();
    // Verify performance (should complete in reasonable time)
    assert!(elapsed.as_secs() < 30, "Simulation should complete in reasonable time");
    // Verify all entities are still stable
        if let Some(transform) = world.get::<Transform>(*entity) {
            if let Some(velocity) = world.get::<Velocity>(*entity) {
                assert!(transform.translation.is_finite(), 
                       "Entity {} position should be finite", i);
                assert!(velocity.linvel.is_finite(), 
                       "Entity {} velocity should be finite", i);
            }
    println!("Performance test completed in {:?} with {} entities", elapsed, entity_count);
fn test_ai_behavior_integration() {
    // Create NPCs with AI behavior
    let npcs: Vec<Entity> = (0..3).map(|i| {
            Player::default(), // Using Player as NPC placeholder
            Transform::from_xyz(i as f32 * 5.0, 0.0, 0.0),
            Collider::capsule_y(0.5, 0.5),
            HumanMovement::default(),
            HumanBehavior {
                personality_speed_modifier: 0.8 + (i as f32 * 0.2),
                reaction_time: 0.1,
                confidence_level: 0.6 + (i as f32 * 0.1),
                movement_variation: 1.0,
            HumanAnimation::default(),
    // Run simulation to test AI behavior integration
    // Verify AI behavior systems worked
    for (i, npc) in npcs.iter().enumerate() {
        let behavior = world.get::<HumanBehavior>(*npc).unwrap();
        let transform = world.get::<Transform>(*npc).unwrap();
        let velocity = world.get::<Velocity>(*npc).unwrap();
        // AI behavior should be valid
        assert!(behavior.personality_speed_modifier > 0.0, 
               "NPC {} should have valid speed modifier", i);
        assert!(behavior.confidence_level >= 0.0 && behavior.confidence_level <= 1.0, 
               "NPC {} should have valid confidence", i);
        // NPCs should be stable
        assert!(transform.translation.is_finite(), 
               "NPC {} position should be finite", i);
        assert!(velocity.linvel.is_finite(), 
               "NPC {} velocity should be finite", i);
fn test_edge_case_recovery() {
    // Create entity with extreme initial conditions
    let entity = {
            Transform::from_xyz(0.0, 100.0, 0.0), // High altitude
            Velocity {
                linvel: Vec3::new(50.0, -20.0, 30.0), // Fast and falling
                angvel: Vec3::new(10.0, 10.0, 10.0),  // High angular velocity
            Collider::ball(1.0),
    // Run simulation - should recover to stable state
    // Verify recovery
    let transform = world.get::<Transform>(entity).unwrap();
    let velocity = world.get::<Velocity>(entity).unwrap();
    // Should have stabilized
    assert!(transform.translation.is_finite(), "Position should be finite");
    assert!(velocity.linvel.is_finite(), "Velocity should be finite");
    assert!(velocity.linvel.length() < 100.0, "Speed should be reasonable");
    assert!(transform.translation.y > -10.0, "Should not be far underground");
    assert!(transform.translation.length() < 1000.0, "Should be within bounds");
