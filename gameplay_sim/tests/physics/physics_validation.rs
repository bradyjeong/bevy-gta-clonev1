#![cfg(feature = "heavy_tests")]
// Physics validation tests using property-based testing
use proptest::prelude::*;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use game_core::prelude::*;
use gameplay_sim::systems::physics_utils::PhysicsUtilities;
use crate::utils::*;

// Property-based test strategies
fn velocity_strategy() -> impl Strategy<Value = Vec3> {
    prop::array::uniform3(-200.0f32..200.0f32)
        .prop_map(|[x, y, z]| Vec3::new(x, y, z))
}
fn mass_strategy() -> impl Strategy<Value = f32> {
    100.0f32..10000.0f32
fn force_strategy() -> impl Strategy<Value = Vec3> {
    prop::array::uniform3(-50000.0f32..50000.0f32)
fn position_strategy() -> impl Strategy<Value = Vec3> {
    prop::array::uniform3(-5000.0f32..5000.0f32)
proptest! {
    #[test]
    fn test_velocity_validation_properties(
        velocity in velocity_strategy()
    ) {
        let config = GameConfig::default();
        let mut vel = Velocity {
            linvel: velocity,
            angvel: Vec3::new(0.0, 0.0, 0.0),
        };
        
        PhysicsUtilities::validate_velocity(&mut vel, &config);
        // Velocity should be finite and within limits
        prop_assert!(vel.linvel.is_finite());
        prop_assert!(vel.linvel.length() <= config.physics.max_velocity);
        prop_assert!(vel.angvel.is_finite());
        prop_assert!(vel.angvel.length() <= config.physics.max_angular_velocity);
    }
    
    fn test_mass_validation_properties(
        mass in mass_strategy()
        // Test mass validation
        let result = PhysicsValidator::validate_mass(mass, config.physics.min_mass, config.physics.max_mass);
        if mass >= config.physics.min_mass && mass <= config.physics.max_mass {
            prop_assert!(result.is_ok());
        } else {
            prop_assert!(result.is_err());
        }
    fn test_force_application_properties(
        force in force_strategy(),
        initial_velocity in velocity_strategy()
        let mut velocity = Velocity {
            linvel: initial_velocity,
            angvel: Vec3::ZERO,
        let dt = 1.0 / 60.0; // Fixed timestep
        let max_force = 10000.0;
        PhysicsUtilities::apply_force_safe(&mut velocity, force, Vec3::ZERO, dt, max_force);
        // Velocity should remain finite
        prop_assert!(velocity.linvel.is_finite());
        prop_assert!(velocity.angvel.is_finite());
        // Force should be clamped
        let expected_force = force.clamp_length_max(max_force);
        if expected_force.is_finite() && expected_force.length() > 0.01 {
            let expected_velocity = initial_velocity + expected_force * dt;
            prop_assert!(vec3_equals(velocity.linvel, expected_velocity, 0.001));
    fn test_position_bounds_properties(
        position in position_strategy(),
        let transform = Transform::from_translation(position);
        let initial_velocity = vel.linvel;
        PhysicsUtilities::apply_world_bounds(&mut vel, &transform, &config);
        let bounds = config.physics.max_world_coord;
        // If position is outside bounds, velocity should be corrected
        if transform.translation.x > bounds {
            prop_assert!(vel.linvel.x <= initial_velocity.x);
        } else if transform.translation.x < -bounds {
            prop_assert!(vel.linvel.x >= initial_velocity.x);
        if transform.translation.z > bounds {
            prop_assert!(vel.linvel.z <= initial_velocity.z);
        } else if transform.translation.z < -bounds {
            prop_assert!(vel.linvel.z >= initial_velocity.z);
#[test]
fn test_vehicle_mass_clamping() {
    let mut app = create_test_app();
    let config = GameConfig::default();
    // Test various mass values
    let test_masses = [
        0.0,      // Invalid (too low)
        50.0,     // Below minimum
        1000.0,   // Valid
        5000.0,   // Valid
        100000.0, // Above maximum
        f32::INFINITY, // Invalid
        f32::NAN, // Invalid
    ];
    for mass in test_masses {
        if mass >= config.physics.min_mass && mass <= config.physics.max_mass && mass.is_finite() {
            assert!(result.is_ok(), "Mass {} should be valid", mass);
            assert!(result.is_err(), "Mass {} should be invalid", mass);
fn test_velocity_clamping_edge_cases() {
    let test_cases = [
        (Vec3::new(f32::INFINITY, 0.0, 0.0), "Infinite X velocity"),
        (Vec3::new(0.0, f32::INFINITY, 0.0), "Infinite Y velocity"),
        (Vec3::new(0.0, 0.0, f32::INFINITY), "Infinite Z velocity"),
        (Vec3::new(f32::NAN, 0.0, 0.0), "NaN X velocity"),
        (Vec3::new(0.0, f32::NAN, 0.0), "NaN Y velocity"),
        (Vec3::new(0.0, 0.0, f32::NAN), "NaN Z velocity"),
        (Vec3::new(1000.0, 0.0, 0.0), "High X velocity"),
        (Vec3::new(0.0, 1000.0, 0.0), "High Y velocity"),
        (Vec3::new(0.0, 0.0, 1000.0), "High Z velocity"),
    for (velocity, description) in test_cases {
        assert!(vel.linvel.is_finite(), "{}: velocity should be finite", description);
        assert!(vel.linvel.length() <= config.physics.max_velocity, 
                "{}: velocity should be clamped", description);
fn test_ground_collision_validation() {
    // Test vehicle at various heights
    let test_heights = [-5.0, -1.0, -0.1, 0.0, 0.1, 1.0, 5.0];
    for height in test_heights {
        let transform = Transform::from_xyz(0.0, height, 0.0);
            linvel: Vec3::new(0.0, -10.0, 0.0), // Falling
        PhysicsUtilities::apply_ground_collision(&mut velocity, &transform, 0.1, 1.0);
        if height < 0.1 {
            // Should stop downward movement and add upward force
            assert!(velocity.linvel.y >= 0.0, "Should stop falling when below ground");
fn test_drag_force_calculation() {
    let velocities = [
        Vec3::new(0.0, 0.0, 0.0),    // Stationary
        Vec3::new(10.0, 0.0, 0.0),   // Moving in X
        Vec3::new(0.0, 10.0, 0.0),   // Moving in Y
        Vec3::new(0.0, 0.0, 10.0),   // Moving in Z
        Vec3::new(10.0, 10.0, 10.0), // Moving in all directions
    for velocity in velocities {
        let vel = Velocity {
        let drag = PhysicsUtilities::calculate_drag_force(&vel, 0.3, 1.225, 2.0);
        if velocity.length() > 0.0 {
            // Drag should oppose motion
            assert!(drag.dot(velocity) <= 0.0, "Drag should oppose motion");
            
            // Drag magnitude should be proportional to velocity squared
            let expected_magnitude = 0.5 * 1.225 * 0.3 * 2.0 * velocity.length_squared();
            assert!(f32_equals(drag.length(), expected_magnitude, 0.001),
                    "Drag magnitude should match formula");
            // No drag when stationary
            assert!(drag.length() < 0.001, "No drag when stationary");
fn test_realistic_vehicle_component_validation() {
    // Create vehicle with extreme values
    let vehicle_entity = {
        let mut world = app.world_mut();
        world.spawn((
            Car::default(),
            ActiveEntity,
            Transform::from_xyz(0.0, 1.0, 0.0),
            GlobalTransform::default(),
            Velocity::default(),
            RigidBody::Dynamic,
            Collider::cuboid(2.0, 1.0, 4.0),
            RealisticVehicle {
                physics_enabled: true,
                last_update_time: 0.0,
            },
            VehicleDynamics {
                speed: f32::INFINITY, // Invalid
                total_mass: -100.0,   // Invalid
                center_of_gravity: Vec3::new(f32::NAN, 0.0, 0.0), // Invalid
                drag_coefficient: -1.0, // Invalid
                downforce_coefficient: f32::INFINITY, // Invalid
                frontal_area: 0.0, // Invalid
                aerodynamic_force: Vec3::ZERO,
                weight_transfer: Vec3::ZERO,
            EnginePhysics {
                max_torque: -1000.0, // Invalid
                max_rpm: 0.0, // Invalid
                idle_rpm: f32::NAN, // Invalid
                current_rpm: f32::INFINITY, // Invalid
                current_gear: -5, // Invalid
                throttle_input: 2.0, // Invalid
                brake_input: -1.0, // Invalid
                gear_ratios: vec![f32::NAN, f32::INFINITY], // Invalid
                differential_ratio: 0.0, // Invalid
                power_curve: vec![f32::NAN, -1.0], // Invalid
            VehicleSuspension {
                force: f32::INFINITY, // Invalid
                ground_contact: true,
            TirePhysics {
                normal_force: -1000.0, // Invalid
                longitudinal_force: f32::NAN, // Invalid
                lateral_force: f32::INFINITY, // Invalid
                slip_ratio: 5.0, // Invalid
                slip_angle: f32::NAN, // Invalid
                tire_temperature: -100.0, // Invalid
                dry_grip: 2.0, // Invalid
                lateral_grip: -1.0, // Invalid
                rolling_resistance: f32::INFINITY, // Invalid
                wear_level: 2.0, // Invalid
        )).id()
    };
    // Run simulation - should not crash due to validation
    run_simulation_duration(&mut app, 1.0);
    // Check that values were clamped/corrected
    let world = app.world();
    let dynamics = world.get::<VehicleDynamics>(vehicle_entity).unwrap();
    let engine = world.get::<EnginePhysics>(vehicle_entity).unwrap();
    let tire_physics = world.get::<TirePhysics>(vehicle_entity).unwrap();
    // All values should be finite and within reasonable ranges
    assert!(dynamics.speed.is_finite(), "Speed should be finite");
    assert!(dynamics.total_mass > 0.0, "Mass should be positive");
    assert!(dynamics.center_of_gravity.is_finite(), "Center of gravity should be finite");
    assert!(dynamics.drag_coefficient >= 0.0, "Drag coefficient should be non-negative");
    assert!(engine.max_torque >= 0.0, "Max torque should be non-negative");
    assert!(engine.max_rpm > 0.0, "Max RPM should be positive");
    assert!(engine.idle_rpm.is_finite(), "Idle RPM should be finite");
    assert!(engine.current_rpm.is_finite(), "Current RPM should be finite");
    assert!(engine.current_gear >= 0, "Gear should be non-negative");
    assert!(engine.throttle_input >= 0.0 && engine.throttle_input <= 1.0, "Throttle should be 0-1");
    assert!(engine.brake_input >= 0.0 && engine.brake_input <= 1.0, "Brake should be 0-1");
    assert!(tire_physics.normal_force >= 0.0, "Normal force should be non-negative");
    assert!(tire_physics.longitudinal_force.is_finite(), "Longitudinal force should be finite");
    assert!(tire_physics.lateral_force.is_finite(), "Lateral force should be finite");
    assert!(tire_physics.slip_ratio >= -1.0 && tire_physics.slip_ratio <= 1.0, "Slip ratio should be -1 to 1");
    assert!(tire_physics.slip_angle.is_finite(), "Slip angle should be finite");
    assert!(tire_physics.tire_temperature >= 0.0, "Tire temperature should be non-negative");
    assert!(tire_physics.dry_grip >= 0.0 && tire_physics.dry_grip <= 2.0, "Dry grip should be 0-2");
fn test_physics_stability_under_extreme_conditions() {
    let (player_entity, vehicle_entity, _) = setup_test_scene(&mut app);
    // Apply extreme forces
    {
        // Extreme velocity
        if let Some(mut velocity) = world.get_mut::<Velocity>(vehicle_entity) {
            velocity.linvel = Vec3::new(1000.0, 1000.0, 1000.0);
            velocity.angvel = Vec3::new(100.0, 100.0, 100.0);
        // Extreme position
        if let Some(mut transform) = world.get_mut::<Transform>(player_entity) {
            transform.translation = Vec3::new(10000.0, 10000.0, 10000.0);
    // Run simulation - should stabilize due to safety systems
    run_simulation_duration(&mut app, 2.0);
    // Check that entities are stable
    let player_transform = world.get::<Transform>(player_entity).unwrap();
    let player_velocity = world.get::<Velocity>(player_entity).unwrap();
    let vehicle_velocity = world.get::<Velocity>(vehicle_entity).unwrap();
    // All values should be finite and reasonable
    assert!(player_transform.translation.is_finite(), "Player position should be finite");
    assert!(player_velocity.linvel.is_finite(), "Player velocity should be finite");
    assert!(vehicle_velocity.linvel.is_finite(), "Vehicle velocity should be finite");
    // Velocities should be clamped to reasonable values
    assert!(player_velocity.linvel.length() < 200.0, "Player velocity should be clamped");
    assert!(vehicle_velocity.linvel.length() < 200.0, "Vehicle velocity should be clamped");
