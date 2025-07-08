#![cfg(feature = "heavy_tests")]
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

#[test]
fn test_supercar_gear_shifting() {
    let mut app = create_test_app();
    let supercar_entity = create_test_supercar(&mut app);
    
    // Apply constant acceleration to trigger gear shifts
    let initial_gear = {
        let world = app.world();
        world.get::<SuperCar>(supercar_entity).unwrap().gear
    };
    
    // Set up control manager for acceleration
    let mut control_manager = ControlManager::new();
    control_manager.set_control_value(ControlAction::Accelerate, 1.0);
    app.insert_resource(control_manager);
    
    // Run simulation for extended period
    run_simulation_duration(&mut app, 5.0);
    
    let final_gear = {
        let world = app.world();
        world.get::<SuperCar>(supercar_entity).unwrap().gear
    };
    
    assert!(final_gear > initial_gear, "Supercar should shift gears during acceleration");
    assert!(final_gear <= 7, "Supercar should not exceed 7th gear");
}

#[test]
fn test_supercar_rpm_management() {
    let mut app = create_test_app();
    let supercar_entity = create_test_supercar(&mut app);
    
    // Set up control manager for acceleration
    let mut control_manager = ControlManager::new();
    control_manager.set_control_value(ControlAction::Accelerate, 1.0);
    app.insert_resource(control_manager);
    
    // Track RPM changes
    let mut rpm_history = Vec::new();
    for _ in 0..300 { // 5 seconds at 60 FPS
        let world = app.world();
        let supercar = world.get::<SuperCar>(supercar_entity).unwrap();
        rpm_history.push(supercar.rpm);
        
        app.update();
    }
    
    // Verify RPM stays within limits
    for rpm in &rpm_history {
        assert!(*rpm >= 800.0, "RPM should not go below idle");
        assert!(*rpm <= 8000.0, "RPM should not exceed redline");
    }
}

#[test]
fn test_supercar_active_aerodynamics() {
    let mut app = create_test_app();
    let supercar_entity = create_test_supercar(&mut app);
    
    // Set high speed to trigger active aero
    {
        let mut world = app.world_mut();
        if let Some(mut velocity) = world.get_mut::<Velocity>(supercar_entity) {
            velocity.linvel = Vec3::new(0.0, 0.0, 50.0); // Very high speed
        }
    }
    
    run_simulation_duration(&mut app, 1.0);
    
    // Check active aerodynamics engagement
    let world = app.world();
    let supercar = world.get::<SuperCar>(supercar_entity).unwrap();
    assert!(supercar.active_aero_deployed, "Active aerodynamics should deploy at high speed");
    assert!(supercar.downforce > 0.0, "Downforce should be generated");
}

#[test]
fn test_supercar_stability_control() {
    let mut app = create_test_app();
    let supercar_entity = create_test_supercar(&mut app);
    
    // Enable stability control
    {
        let mut world = app.world_mut();
        if let Some(mut supercar) = world.get_mut::<SuperCar>(supercar_entity) {
            supercar.stability_control = true;
        }
    }
    
    // Set up control manager for aggressive steering
    let mut control_manager = ControlManager::new();
    control_manager.set_control_value(ControlAction::Steer, 1.0); // Full steering
    app.insert_resource(control_manager);
    
    // Apply aggressive steering at high speed
    {
        let mut world = app.world_mut();
        if let Some(mut velocity) = world.get_mut::<Velocity>(supercar_entity) {
            velocity.linvel = Vec3::new(0.0, 0.0, 30.0); // High speed
        }
    }
    
    run_simulation_duration(&mut app, 1.0);
    
    // Check that stability control limits excessive rotation
    let world = app.world();
    let velocity = world.get::<Velocity>(supercar_entity).unwrap();
    assert!(velocity.angvel.y.abs() < 5.0, "Stability control should limit excessive rotation");
}
