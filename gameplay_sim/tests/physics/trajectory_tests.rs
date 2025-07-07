// Trajectory tests for deterministic physics validation
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use game_core::prelude::*;
use gameplay_sim::systems::input::{ControlManager, ControlAction};
use crate::utils::*;
use std::path::Path;

#[test]
fn test_vehicle_straight_line_trajectory() {
    let mut app = create_test_app();
    let (_, vehicle_entity, _) = setup_test_scene(&mut app);
    
    // Set up constant acceleration
    let mut control_manager = ControlManager::new();
    control_manager.set_control_value(ControlAction::Accelerate, 0.5);
    app.insert_resource(control_manager);
    // Capture trajectory
    let trajectory = capture_vehicle_trajectory(&mut app, vehicle_entity, 3.0);
    // Verify trajectory properties
    assert!(trajectory.len() > 0, "Should capture trajectory points");
    // Check that vehicle moves in straight line (minimal lateral movement)
    let start_pos = trajectory[0].1;
    let end_pos = trajectory[trajectory.len() - 1].1;
    // Should move primarily in Z direction (forward)
    assert!((end_pos.z - start_pos.z).abs() > 1.0, "Vehicle should move forward");
    assert!((end_pos.x - start_pos.x).abs() < 0.5, "Vehicle should not drift sideways");
    // Velocity should increase over time
    let start_speed = trajectory[0].2.length();
    let end_speed = trajectory[trajectory.len() - 1].2.length();
    assert!(end_speed > start_speed, "Vehicle should accelerate");
    // Save trajectory for golden frame comparison
    let _ = save_trajectory_csv("target/test_straight_line_trajectory.csv", &trajectory);
}
fn test_vehicle_turning_trajectory() {
    // Set up acceleration and steering
    control_manager.set_control_value(ControlAction::Accelerate, 0.3);
    control_manager.set_control_value(ControlAction::Steer, 0.5); // Turn right
    let trajectory = capture_vehicle_trajectory(&mut app, vehicle_entity, 4.0);
    // Verify turning trajectory
    // Should have significant X movement (turning)
    assert!((end_pos.x - start_pos.x).abs() > 1.0, "Vehicle should turn");
    // Should follow curved path
    let mut max_x = start_pos.x;
    let mut min_x = start_pos.x;
    for (_, pos, _) in &trajectory {
        max_x = max_x.max(pos.x);
        min_x = min_x.min(pos.x);
    }
    assert!(max_x - min_x > 1.0, "Vehicle should follow curved path");
    // Save trajectory for analysis
    let _ = save_trajectory_csv("target/test_turning_trajectory.csv", &trajectory);
fn test_vehicle_braking_trajectory() {
    // First phase: accelerate
    control_manager.set_control_value(ControlAction::Accelerate, 1.0);
    // Accelerate for 2 seconds
    run_simulation_duration(&mut app, 2.0);
    // Second phase: brake
    control_manager.set_control_value(ControlAction::Brake, 1.0);
    // Capture braking trajectory
    let braking_trajectory = capture_vehicle_trajectory(&mut app, vehicle_entity, 3.0);
    // Verify braking behavior
    assert!(braking_trajectory.len() > 0, "Should capture braking trajectory");
    // Speed should decrease over time
    let start_speed = braking_trajectory[0].2.length();
    let end_speed = braking_trajectory[braking_trajectory.len() - 1].2.length();
    assert!(end_speed < start_speed, "Vehicle should decelerate when braking");
    // Should stop or significantly slow down
    assert!(end_speed < start_speed * 0.5, "Vehicle should slow down significantly");
    // Save trajectory
    let _ = save_trajectory_csv("target/test_braking_trajectory.csv", &braking_trajectory);
fn test_supercar_launch_trajectory() {
    let supercar_entity = create_test_supercar(&mut app);
    // Set up launch conditions
    control_manager.set_control_value(ControlAction::Turbo, 1.0);
    // Enable launch control
    {
        let mut world = app.world_mut();
        if let Some(mut supercar) = world.get_mut::<SuperCar>(supercar_entity) {
            supercar.launch_control_engaged = true;
            supercar.is_timing_launch = true;
        }
    // Capture launch trajectory
    let launch_trajectory = capture_vehicle_trajectory(&mut app, supercar_entity, 5.0);
    // Verify launch characteristics
    assert!(launch_trajectory.len() > 0, "Should capture launch trajectory");
    // Should have smooth acceleration without excessive wheel spin
    let mut max_acceleration = 0.0;
    for i in 1..launch_trajectory.len() {
        let dt = launch_trajectory[i].0 - launch_trajectory[i-1].0;
        let dv = launch_trajectory[i].2.length() - launch_trajectory[i-1].2.length();
        let acceleration = dv / dt;
        max_acceleration = max_acceleration.max(acceleration);
    assert!(max_acceleration > 0.0, "Should have positive acceleration");
    assert!(max_acceleration < 50.0, "Launch control should limit acceleration");
    let _ = save_trajectory_csv("target/test_supercar_launch_trajectory.csv", &launch_trajectory);
fn test_trajectory_reproducibility() {
    // Test that identical conditions produce identical trajectories
    let mut app1 = create_test_app();
    let mut app2 = create_test_app();
    let vehicle1 = create_test_supercar(&mut app1);
    let vehicle2 = create_test_supercar(&mut app2);
    // Identical input conditions
    let control_manager = {
        let mut cm = ControlManager::new();
        cm.set_control_value(ControlAction::Accelerate, 0.7);
        cm.set_control_value(ControlAction::Steer, 0.3);
        cm
    };
    app1.insert_resource(control_manager.clone());
    app2.insert_resource(control_manager);
    // Capture trajectories
    let trajectory1 = capture_vehicle_trajectory(&mut app1, vehicle1, 2.0);
    let trajectory2 = capture_vehicle_trajectory(&mut app2, vehicle2, 2.0);
    // Trajectories should be identical (within epsilon)
    assert_eq!(trajectory1.len(), trajectory2.len(), "Trajectories should have same length");
    for i in 0..trajectory1.len() {
        let (t1, pos1, vel1) = trajectory1[i];
        let (t2, pos2, vel2) = trajectory2[i];
        
        assert!(f32_equals(t1, t2, 0.001), "Times should match");
        assert!(vec3_equals(pos1, pos2, 0.001), "Positions should match at step {}", i);
        assert!(vec3_equals(vel1, vel2, 0.001), "Velocities should match at step {}", i);
fn test_compare_with_golden_trajectory() {
    // Test against known good trajectory data
    let vehicle_entity = create_test_supercar(&mut app);
    // Set up specific test conditions
    // Capture current trajectory
    let current_trajectory = capture_vehicle_trajectory(&mut app, vehicle_entity, 3.0);
    // Save current trajectory as reference if golden file doesn't exist
    let golden_path = "target/golden_trajectory.csv";
    if !Path::new(golden_path).exists() {
        let _ = save_trajectory_csv(golden_path, &current_trajectory);
        println!("Created golden trajectory file: {}", golden_path);
        return;
    // Load golden trajectory
    if let Ok(golden_data) = load_golden_csv(golden_path) {
        // Compare with golden data
        let tolerance = 0.1; // 10cm tolerance
        for (i, (time, pos, vel)) in current_trajectory.iter().enumerate() {
            if i < golden_data.len() {
                let golden_row = &golden_data[i];
                if golden_row.len() >= 7 {
                    let golden_time = golden_row[0];
                    let golden_pos = Vec3::new(golden_row[1], golden_row[2], golden_row[3]);
                    let golden_vel = Vec3::new(golden_row[4], golden_row[5], golden_row[6]);
                    
                    assert!(f32_equals(*time, golden_time, 0.001), 
                           "Time mismatch at step {}: {} vs {}", i, time, golden_time);
                    assert!(vec3_equals(*pos, golden_pos, tolerance), 
                           "Position mismatch at step {}: {:?} vs {:?}", i, pos, golden_pos);
                    assert!(vec3_equals(*vel, golden_vel, tolerance), 
                           "Velocity mismatch at step {}: {:?} vs {:?}", i, vel, golden_vel);
                }
            }
fn test_player_movement_trajectory() {
    let (player_entity, _, _) = setup_test_scene(&mut app);
    // Set up player movement
    control_manager.set_control_value(ControlAction::Accelerate, 1.0); // Forward
    // Capture player trajectory
    let player_trajectory = capture_vehicle_trajectory(&mut app, player_entity, 2.0);
    // Verify player movement
    assert!(player_trajectory.len() > 0, "Should capture player trajectory");
    let start_pos = player_trajectory[0].1;
    let end_pos = player_trajectory[player_trajectory.len() - 1].1;
    // Player should move
    assert!((end_pos - start_pos).length() > 0.1, "Player should move with input");
    let _ = save_trajectory_csv("target/test_player_trajectory.csv", &player_trajectory);
fn test_multiple_vehicle_trajectories() {
    // Create multiple vehicles
    let vehicle1 = create_test_supercar(&mut app);
    let vehicle2 = {
        world.spawn((
            Car::default(),
            ActiveEntity,
            Transform::from_xyz(5.0, 1.0, 0.0), // Different starting position
            GlobalTransform::default(),
            Velocity::default(),
            RigidBody::Dynamic,
            Collider::cuboid(2.0, 1.0, 4.0),
            RealisticVehicle::default(),
            VehicleDynamics::default(),
            EnginePhysics::default(),
            VehicleSuspension::default(),
            TirePhysics::default(),
        )).id()
    // Different inputs for each vehicle
    control_manager.set_control_value(ControlAction::Accelerate, 0.8);
    control_manager.set_control_value(ControlAction::Steer, 0.2);
    let trajectory1 = capture_vehicle_trajectory(&mut app, vehicle1, 3.0);
    let trajectory2 = capture_vehicle_trajectory(&mut app, vehicle2, 3.0);
    // Verify both vehicles move
    assert!(trajectory1.len() > 0, "Vehicle 1 should have trajectory");
    assert!(trajectory2.len() > 0, "Vehicle 2 should have trajectory");
    // Starting positions should be different
    let start1 = trajectory1[0].1;
    let start2 = trajectory2[0].1;
    assert!((start1 - start2).length() > 1.0, "Vehicles should start at different positions");
    // Both should move
    let end1 = trajectory1[trajectory1.len() - 1].1;
    let end2 = trajectory2[trajectory2.len() - 1].1;
    assert!((end1 - start1).length() > 0.5, "Vehicle 1 should move");
    assert!((end2 - start2).length() > 0.5, "Vehicle 2 should move");
    // Save trajectories
    let _ = save_trajectory_csv("target/test_multi_vehicle1_trajectory.csv", &trajectory1);
    let _ = save_trajectory_csv("target/test_multi_vehicle2_trajectory.csv", &trajectory2);
fn test_trajectory_physics_conservation() {
    // No input - test conservation of momentum
    app.insert_resource(ControlManager::new());
    // Give initial velocity
        if let Some(mut velocity) = world.get_mut::<Velocity>(vehicle_entity) {
            velocity.linvel = Vec3::new(0.0, 0.0, 10.0); // 10 m/s forward
    let trajectory = capture_vehicle_trajectory(&mut app, vehicle_entity, 5.0);
    // Verify momentum conservation (with some energy loss due to friction/drag)
    let initial_speed = trajectory[0].2.length();
    let final_speed = trajectory[trajectory.len() - 1].2.length();
    assert!(initial_speed > 0.0, "Should have initial speed");
    assert!(final_speed > 0.0, "Should maintain some speed");
    assert!(final_speed <= initial_speed, "Speed should decrease due to friction");
    // Should maintain forward direction
    let initial_dir = trajectory[0].2.normalize();
    let final_dir = trajectory[trajectory.len() - 1].2.normalize();
    assert!(initial_dir.dot(final_dir) > 0.8, "Should maintain general direction");
    let _ = save_trajectory_csv("target/test_conservation_trajectory.csv", &trajectory);
