// Vehicle physics integration tests
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use game_core::prelude::*;
use gameplay_sim::prelude::*;
use crate::utils::*;

#[test]
fn test_realistic_vehicle_physics_integration() {
    let mut app = create_test_app();
    let (_, vehicle_entity, _) = setup_test_scene(&mut app);
    
    // Initialize control manager
    app.insert_resource(ControlManager::new());
    // Run simulation for 1 second
    run_simulation_duration(&mut app, 1.0);
    // Verify vehicle still exists and has valid physics state
    let world = app.world();
    let transform = world.get::<Transform>(vehicle_entity).unwrap();
    let velocity = world.get::<Velocity>(vehicle_entity).unwrap();
    // Validate physics state
    assert!(PhysicsValidator::validate_velocity(velocity, 100.0).is_ok());
    assert!(PhysicsValidator::validate_position(transform, 1000.0).is_ok());
    // Vehicle should have settled due to gravity
    assert!(transform.translation.y < 1.0, "Vehicle should settle on ground");
    assert!(velocity.linvel.length() < 0.1, "Vehicle should be at rest");
}
fn test_vehicle_acceleration_physics() {
    // Set up control manager with acceleration input
    let mut control_manager = ControlManager::new();
    control_manager.set_control_value(ControlAction::Accelerate, 1.0);
    app.insert_resource(control_manager);
    // Get initial state
    let initial_velocity = {
        let world = app.world();
        world.get::<Velocity>(vehicle_entity).unwrap().linvel.length()
    };
    // Run simulation for 2 seconds
    run_simulation_duration(&mut app, 2.0);
    // Check that vehicle accelerated
    let final_velocity = world.get::<Velocity>(vehicle_entity).unwrap().linvel.length();
    assert!(final_velocity > initial_velocity, "Vehicle should accelerate with input");
    assert!(final_velocity < 50.0, "Vehicle should not exceed reasonable speed");
fn test_vehicle_braking_physics() {
    // First, give vehicle some initial velocity
    {
        let mut world = app.world_mut();
        if let Some(mut velocity) = world.get_mut::<Velocity>(vehicle_entity) {
            velocity.linvel = Vec3::new(0.0, 0.0, 20.0); // 20 m/s forward
        }
    }
    // Set up control manager with braking input
    control_manager.set_control_value(ControlAction::Brake, 1.0);
    let initial_speed = {
    // Check that vehicle decelerated
    let final_speed = world.get::<Velocity>(vehicle_entity).unwrap().linvel.length();
    assert!(final_speed < initial_speed, "Vehicle should decelerate with braking");
fn test_vehicle_steering_physics() {
    // Give vehicle forward velocity and steering input
            velocity.linvel = Vec3::new(0.0, 0.0, 10.0); // 10 m/s forward
    control_manager.set_control_value(ControlAction::Steer, 0.5); // Turn right
    let initial_position = {
        world.get::<Transform>(vehicle_entity).unwrap().translation
    // Check that vehicle turned (x position should change)
    let final_position = world.get::<Transform>(vehicle_entity).unwrap().translation;
    assert!(
        (final_position.x - initial_position.x).abs() > 0.5,
        "Vehicle should turn when steering input is applied"
    );
fn test_vehicle_mass_affects_physics() {
    let mut app1 = create_test_app();
    let mut app2 = create_test_app();
    // Create vehicles with different masses
    let light_vehicle = {
        let mut world = app1.world_mut();
        world.spawn((
            Car::default(),
            ActiveEntity,
            Transform::from_xyz(0.0, 1.0, 0.0),
            GlobalTransform::default(),
            Velocity::default(),
            RigidBody::Dynamic,
            Collider::cuboid(2.0, 1.0, 4.0),
            AdditionalMassProperties::Mass(1000.0), // Light vehicle
            RealisticVehicle::default(),
            VehicleDynamics::default(),
            EnginePhysics::default(),
            VehicleSuspension::default(),
            TirePhysics::default(),
        )).id()
    let heavy_vehicle = {
        let mut world = app2.world_mut();
            AdditionalMassProperties::Mass(3000.0), // Heavy vehicle
    // Apply same acceleration to both
    app1.insert_resource(control_manager.clone());
    app2.insert_resource(control_manager);
    // Run simulation
    run_simulation_duration(&mut app1, 2.0);
    run_simulation_duration(&mut app2, 2.0);
    // Check that lighter vehicle accelerates more
    let light_speed = {
        let world = app1.world();
        world.get::<Velocity>(light_vehicle).unwrap().linvel.length()
    let heavy_speed = {
        let world = app2.world();
        world.get::<Velocity>(heavy_vehicle).unwrap().linvel.length()
        light_speed > heavy_speed,
        "Lighter vehicle should accelerate faster than heavier vehicle"
fn test_vehicle_engine_physics_integration() {
    // Test engine RPM changes with throttle
    let initial_rpm = {
        world.get::<EnginePhysics>(vehicle_entity).unwrap().current_rpm
    let final_rpm = {
    assert!(final_rpm > initial_rpm, "Engine RPM should increase with throttle");
    // Test gear shifting
    let gear = {
        world.get::<EnginePhysics>(vehicle_entity).unwrap().current_gear
    assert!(gear > 0, "Vehicle should be in gear when accelerating");
fn test_tire_physics_ground_contact() {
    // Run simulation to let vehicle settle
    // Check tire physics state
    let tire_physics = world.get::<TirePhysics>(vehicle_entity).unwrap();
    let suspension = world.get::<VehicleSuspension>(vehicle_entity).unwrap();
    assert!(suspension.ground_contact, "Vehicle should have ground contact");
    assert!(tire_physics.normal_force > 0.0, "Tires should have normal force");
fn test_vehicle_aerodynamics() {
    // Give vehicle high speed to test aerodynamics
            velocity.linvel = Vec3::new(0.0, 0.0, 30.0); // 30 m/s (high speed)
    // Check that aerodynamic forces are applied
    let dynamics = world.get::<VehicleDynamics>(vehicle_entity).unwrap();
        dynamics.aerodynamic_force.length() > 0.0,
        "Aerodynamic forces should be applied at high speed"
fn test_vehicle_weight_transfer() {
    // Apply strong acceleration to trigger weight transfer
    // Check weight transfer effects
        dynamics.weight_transfer.length() > 0.0,
        "Weight transfer should occur during acceleration"
