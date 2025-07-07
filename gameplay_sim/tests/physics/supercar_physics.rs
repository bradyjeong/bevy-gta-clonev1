// Supercar physics integration tests
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use game_core::prelude::*;
use gameplay_sim::systems::input::{ControlManager, ControlAction};
use crate::utils::*;

#[test]
fn test_supercar_turbo_system() {
    let mut app = create_test_app();
    let supercar_entity = create_test_supercar(&mut app);
    
    // Set up control manager with turbo input
    let mut control_manager = ControlManager::new();
    control_manager.set_control_value(ControlAction::Turbo, 1.0);
    control_manager.set_control_value(ControlAction::Accelerate, 1.0);
    app.insert_resource(control_manager);
    // Run simulation to build turbo pressure
    run_simulation_duration(&mut app, 2.0);
    // Check turbo system state
    let world = app.world();
    let supercar = world.get::<SuperCar>(supercar_entity).unwrap();
    assert!(supercar.turbo_pressure > 0.0, "Turbo pressure should build up");
    assert!(supercar.turbo_boost, "Turbo should be active");
    assert!(supercar.turbo_stage > 0, "Turbo should have active stages");
}
fn test_supercar_gear_shifting() {
    // Apply constant acceleration to trigger gear shifts
    let initial_gear = {
        let world = app.world();
        world.get::<SuperCar>(supercar_entity).unwrap().gear
    };
    // Run simulation for extended period
    run_simulation_duration(&mut app, 5.0);
    let final_gear = {
    assert!(final_gear > initial_gear, "Supercar should shift gears during acceleration");
    assert!(final_gear <= 7, "Supercar should not exceed 7th gear");
fn test_supercar_rpm_management() {
    // Track RPM changes
    let mut rpm_history = Vec::new();
    for _ in 0..300 { // 5 seconds at 60 FPS
        let supercar = world.get::<SuperCar>(supercar_entity).unwrap();
        rpm_history.push(supercar.rpm);
        
        app.update();
    }
    // Verify RPM stays within limits
    for rpm in &rpm_history {
        assert!(*rpm >= 800.0, "RPM should not go below idle");
        assert!(*rpm <= 8000.0, "RPM should not exceed redline");
    // Verify RPM increases with acceleration
    let initial_rpm = rpm_history[0];
    let final_rpm = rpm_history[rpm_history.len() - 1];
    assert!(final_rpm > initial_rpm, "RPM should increase with acceleration");
fn test_supercar_active_aerodynamics() {
    // Set high speed to trigger active aero
    {
        let mut world = app.world_mut();
        if let Some(mut velocity) = world.get_mut::<Velocity>(supercar_entity) {
            velocity.linvel = Vec3::new(0.0, 0.0, 60.0); // 60 m/s (high speed)
        }
    // Run simulation
    // Check active aero adjustments
    assert!(supercar.rear_wing_angle > 0.0, "Rear wing should deploy at high speed");
    assert!(supercar.downforce > 0.0, "Downforce should be generated");
    assert!(supercar.front_splitter_level > 0.0, "Front splitter should adjust");
fn test_supercar_traction_control() {
    // Enable traction control
        if let Some(mut supercar) = world.get_mut::<SuperCar>(supercar_entity) {
            supercar.traction_control = true;
    // Check traction control effects
    assert!(supercar.current_traction > 0.6, "Traction control should maintain grip");
    assert!(supercar.current_traction <= 1.0, "Traction should not exceed maximum");
fn test_supercar_driving_modes() {
    // Test different driving modes
    let modes = [
        DrivingMode::Comfort,
        DrivingMode::Sport,
        DrivingMode::Track,
        DrivingMode::Custom,
    ];
    for mode in modes {
        // Set driving mode
        {
            let mut world = app.world_mut();
            if let Some(mut supercar) = world.get_mut::<SuperCar>(supercar_entity) {
                supercar.driving_mode = mode.clone();
            }
        let mut control_manager = ControlManager::new();
        control_manager.set_control_value(ControlAction::Accelerate, 1.0);
        app.insert_resource(control_manager);
        // Run short simulation
        run_simulation_duration(&mut app, 1.0);
        // Verify mode affects behavior
        match mode {
            DrivingMode::Comfort => {
                assert!(supercar.current_traction > 0.9, "Comfort mode should prioritize stability");
            DrivingMode::Track => {
                assert!(supercar.current_traction < 0.9, "Track mode should allow more slip");
            _ => {} // Other modes have varying characteristics
fn test_supercar_launch_control() {
    // Enable launch control
            supercar.launch_control_engaged = true;
            supercar.is_timing_launch = true;
    run_simulation_duration(&mut app, 3.0);
    // Check launch control behavior
    let velocity = world.get::<Velocity>(supercar_entity).unwrap();
    // Should have smooth launch without excessive wheel spin
    assert!(velocity.linvel.length() > 0.0, "Vehicle should accelerate");
    assert!(velocity.linvel.length() < 100.0, "Speed should be reasonable");
    // Launch control should limit initial torque
    assert!(supercar.current_traction > 0.7, "Launch control should maintain traction");
fn test_supercar_engine_protection() {
    // Run at high RPM to test engine protection
    // Run extended simulation
    run_simulation_duration(&mut app, 10.0);
    // Check engine protection systems
    // Engine temperature should be managed
    assert!(supercar.engine_temperature <= 1.0, "Engine temperature should be limited");
    // Oil pressure should be stable
    assert!(supercar.oil_pressure > 0.0, "Oil pressure should be maintained");
    assert!(supercar.oil_pressure <= 1.0, "Oil pressure should be within limits");
    // Rev limiter should prevent over-revving
    if supercar.rev_limiter_active {
        assert!(supercar.rpm < supercar.max_rpm, "Rev limiter should prevent over-revving");
fn test_supercar_g_force_calculation() {
    // Apply strong acceleration
    // Check G-force calculations
    // Should have measurable G-forces during acceleration
    assert!(supercar.g_force_longitudinal > 0.0, "Should measure longitudinal G-force");
    assert!(supercar.g_force_longitudinal < 3.0, "G-force should be reasonable");
fn test_supercar_braking_system() {
    // Give initial speed
            velocity.linvel = Vec3::new(0.0, 0.0, 40.0); // 40 m/s
    // Apply brakes
    control_manager.set_control_value(ControlAction::Brake, 1.0);
    let initial_speed = {
        world.get::<Velocity>(supercar_entity).unwrap().linvel.length()
    let final_speed = {
    assert!(final_speed < initial_speed, "Supercar should decelerate with braking");
    // Check that braking is more effective than regular vehicles
    assert!(final_speed < initial_speed * 0.5, "Supercar should have powerful brakes");
fn test_supercar_stability_control() {
    // Enable stability control
            supercar.stability_control = true;
    // Apply aggressive steering at high speed
            velocity.linvel = Vec3::new(0.0, 0.0, 30.0); // High speed
    control_manager.set_control_value(ControlAction::Steer, 1.0); // Full steering
    run_simulation_duration(&mut app, 1.0);
    // Check that stability control limits excessive rotation
    // Angular velocity should be controlled
    assert!(velocity.angvel.y.abs() < 5.0, "Stability control should limit excessive rotation");
