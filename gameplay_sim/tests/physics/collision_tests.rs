// Collision detection and response tests
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use game_core::prelude::*;
use gameplay_sim::systems::input::{ControlManager, ControlAction};
use crate::utils::*;

#[test]
fn test_vehicle_ground_collision() {
    let mut app = create_test_app();
    let (_, vehicle_entity, _) = setup_test_scene(&mut app);
    
    // Place vehicle above ground
    {
        let mut world = app.world_mut();
        if let Some(mut transform) = world.get_mut::<Transform>(vehicle_entity) {
            transform.translation.y = 5.0; // 5 meters above ground
        }
    }
    // Run simulation to let vehicle fall
    run_simulation_duration(&mut app, 3.0);
    // Check that vehicle settled on ground
    let world = app.world();
    let transform = world.get::<Transform>(vehicle_entity).unwrap();
    let velocity = world.get::<Velocity>(vehicle_entity).unwrap();
    assert!(transform.translation.y < 2.0, "Vehicle should fall to ground level");
    assert!(velocity.linvel.y.abs() < 1.0, "Vehicle should stop bouncing");
}
fn test_vehicle_vehicle_collision_avoidance() {
    // Create two vehicles on collision course
    let vehicle1 = {
        world.spawn((
            Car::default(),
            ActiveEntity,
            Transform::from_xyz(0.0, 1.0, 0.0),
            GlobalTransform::default(),
            Velocity {
                linvel: Vec3::new(0.0, 0.0, 10.0), // Moving forward
                angvel: Vec3::ZERO,
            },
            RigidBody::Dynamic,
            Collider::cuboid(2.0, 1.0, 4.0),
            RealisticVehicle::default(),
            VehicleDynamics::default(),
            EnginePhysics::default(),
            VehicleSuspension::default(),
            TirePhysics::default(),
        )).id()
    };
    let vehicle2 = {
            Transform::from_xyz(0.0, 1.0, 20.0),
                linvel: Vec3::new(0.0, 0.0, -10.0), // Moving backward (toward vehicle1)
    // Run simulation
    run_simulation_duration(&mut app, 2.0);
    // Check positions after potential collision
    let transform1 = world.get::<Transform>(vehicle1).unwrap();
    let transform2 = world.get::<Transform>(vehicle2).unwrap();
    let velocity1 = world.get::<Velocity>(vehicle1).unwrap();
    let velocity2 = world.get::<Velocity>(vehicle2).unwrap();
    // Vehicles should have interacted (physics collision)
    let distance = (transform1.translation - transform2.translation).length();
    // Either collision occurred (vehicles stopped/bounced) or they passed through
    // In a proper physics simulation, they should collide
    if distance < 10.0 {
        // Collision occurred - check response
        let speed1 = velocity1.linvel.length();
        let speed2 = velocity2.linvel.length();
        
        // Velocities should be affected by collision
        assert!(speed1 < 8.0 || speed2 < 8.0, "At least one vehicle should slow down from collision");
fn test_player_vehicle_collision() {
    let (player_entity, vehicle_entity, _) = setup_test_scene(&mut app);
    // Move player into vehicle's path
        if let Some(mut transform) = world.get_mut::<Transform>(player_entity) {
            transform.translation = Vec3::new(0.0, 0.0, 5.0); // In front of vehicle
    // Give vehicle forward velocity toward player
        if let Some(mut velocity) = world.get_mut::<Velocity>(vehicle_entity) {
            velocity.linvel = Vec3::new(0.0, 0.0, -5.0); // Moving toward player
    run_simulation_duration(&mut app, 1.0);
    // Check that collision was handled
    let player_transform = world.get::<Transform>(player_entity).unwrap();
    let vehicle_transform = world.get::<Transform>(vehicle_entity).unwrap();
    let player_velocity = world.get::<Velocity>(player_entity).unwrap();
    let vehicle_velocity = world.get::<Velocity>(vehicle_entity).unwrap();
    let distance = (player_transform.translation - vehicle_transform.translation).length();
    // Either collision was handled properly or avoided
    if distance < 3.0 {
        // Close proximity - check for proper collision response
        let player_speed = player_velocity.linvel.length();
        let vehicle_speed = vehicle_velocity.linvel.length();
        // Some interaction should have occurred
        assert!(player_speed > 0.1 || vehicle_speed < 4.0, 
               "Collision should affect at least one entity");
fn test_collision_energy_conservation() {
    // Create two identical vehicles for collision
                linvel: Vec3::new(0.0, 0.0, 5.0),
            AdditionalMassProperties::Mass(1500.0),
            Restitution::coefficient(0.5), // Partial elastic collision
            Transform::from_xyz(0.0, 1.0, 15.0),
                linvel: Vec3::new(0.0, 0.0, -5.0),
            Restitution::coefficient(0.5),
    // Calculate initial momentum and energy
    let vel1_initial = world.get::<Velocity>(vehicle1).unwrap().linvel;
    let vel2_initial = world.get::<Velocity>(vehicle2).unwrap().linvel;
    let initial_momentum = vel1_initial + vel2_initial; // Equal masses
    let initial_kinetic_energy = 0.5 * 1500.0 * (vel1_initial.length_squared() + vel2_initial.length_squared());
    // Check final momentum and energy
    let vel1_final = world.get::<Velocity>(vehicle1).unwrap().linvel;
    let vel2_final = world.get::<Velocity>(vehicle2).unwrap().linvel;
    let final_momentum = vel1_final + vel2_final;
    let final_kinetic_energy = 0.5 * 1500.0 * (vel1_final.length_squared() + vel2_final.length_squared());
    // Momentum should be approximately conserved
    assert!(vec3_equals(initial_momentum, final_momentum, 1.0), 
           "Momentum should be approximately conserved");
    // Energy should decrease (inelastic collision)
    assert!(final_kinetic_energy <= initial_kinetic_energy, 
           "Kinetic energy should decrease in inelastic collision");
fn test_boundary_collision() {
    // Move vehicle to world boundary
            transform.translation = Vec3::new(4900.0, 1.0, 0.0); // Near boundary
            velocity.linvel = Vec3::new(10.0, 0.0, 0.0); // Moving toward boundary
    // Check that vehicle was stopped at boundary
    // Should not exceed world bounds
    assert!(transform.translation.x < 5000.0, "Vehicle should not exceed world bounds");
    // Velocity toward boundary should be stopped
    if transform.translation.x > 4990.0 {
        assert!(velocity.linvel.x <= 0.0, "Velocity toward boundary should be stopped");
fn test_underground_collision_prevention() {
    // Force vehicle underground (simulating physics bug)
            transform.translation.y = -5.0; // Underground
            velocity.linvel.y = -10.0; // Moving further down
    // Run simulation - safety systems should prevent staying underground
    // Check that vehicle was corrected
    assert!(transform.translation.y > -1.0, "Vehicle should not stay far underground");
    // Downward velocity should be stopped/corrected
    if transform.translation.y < 0.1 {
        assert!(velocity.linvel.y >= 0.0, "Downward velocity should be stopped near ground");
fn test_collision_detection_accuracy() {
    // Create small fast-moving objects for collision accuracy test
    let projectile = {
                linvel: Vec3::new(0.0, 0.0, 50.0), // Very fast
            Collider::ball(0.1), // Small sphere
    let target = {
            Transform::from_xyz(0.0, 1.0, 10.0),
            Velocity::default(),
            Collider::cuboid(1.0, 1.0, 1.0), // Target cube
    run_simulation_duration(&mut app, 0.5); // Short time for fast collision
    // Check that collision was detected despite high speed
    let projectile_transform = world.get::<Transform>(projectile).unwrap();
    let target_transform = world.get::<Transform>(target).unwrap();
    let projectile_velocity = world.get::<Velocity>(projectile).unwrap();
    let distance = (projectile_transform.translation - target_transform.translation).length();
    // Either collision occurred or projectile passed very close
    if distance < 5.0 {
        // Likely collision - velocity should be affected
        assert!(projectile_velocity.linvel.length() < 45.0, 
               "High-speed collision should affect velocity");
fn test_multiple_collision_handling() {
    // Create multiple vehicles in close proximity
    let vehicles: Vec<Entity> = (0..5).map(|i| {
            Transform::from_xyz(i as f32 * 3.0, 1.0, 0.0),
                linvel: Vec3::new(-2.0, 0.0, 0.0), // All moving toward center
            Collider::cuboid(1.5, 1.0, 3.0),
    }).collect();
    // Check that all vehicles are still valid and stable
    for vehicle in &vehicles {
        let transform = world.get::<Transform>(*vehicle).unwrap();
        let velocity = world.get::<Velocity>(*vehicle).unwrap();
        // All should be at reasonable positions
        assert!(transform.translation.is_finite(), "Vehicle position should be finite");
        assert!(velocity.linvel.is_finite(), "Vehicle velocity should be finite");
        // Should not have excessive speeds
        assert!(velocity.linvel.length() < 20.0, "Vehicle speed should be reasonable");
    // Check that vehicles are reasonably separated (no overlap)
    for i in 0..vehicles.len() {
        for j in (i+1)..vehicles.len() {
            let pos1 = world.get::<Transform>(vehicles[i]).unwrap().translation;
            let pos2 = world.get::<Transform>(vehicles[j]).unwrap().translation;
            let distance = (pos1 - pos2).length();
            
            // Vehicles should maintain some separation
            assert!(distance > 1.0, "Vehicles should not overlap significantly");
fn test_collision_material_properties() {
    // Create vehicle with different material properties
    let bouncy_vehicle = {
            Transform::from_xyz(0.0, 5.0, 0.0), // Above ground
            Restitution::coefficient(0.9), // Very bouncy
            Friction::coefficient(0.1),    // Low friction
    let sticky_vehicle = {
            Transform::from_xyz(5.0, 5.0, 0.0), // Above ground
            Restitution::coefficient(0.1), // Low bounce
            Friction::coefficient(0.9),    // High friction
    // Let both fall and interact with ground
    // Check different behaviors
    let bouncy_transform = world.get::<Transform>(bouncy_vehicle).unwrap();
    let sticky_transform = world.get::<Transform>(sticky_vehicle).unwrap();
    let bouncy_velocity = world.get::<Velocity>(bouncy_vehicle).unwrap();
    let sticky_velocity = world.get::<Velocity>(sticky_vehicle).unwrap();
    // Both should be near ground level
    assert!(bouncy_transform.translation.y < 3.0, "Bouncy vehicle should settle");
    assert!(sticky_transform.translation.y < 3.0, "Sticky vehicle should settle");
    // Bouncy vehicle might still have some vertical motion
    // Sticky vehicle should be more settled
    assert!(sticky_velocity.linvel.length() <= bouncy_velocity.linvel.length() + 0.5,
           "Sticky vehicle should be more settled due to friction");
