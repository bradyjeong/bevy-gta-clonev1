#![cfg(feature = "heavy_tests")]
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use game_core::prelude::*;
use gameplay_sim::tests::utils::*;

#[test]
fn test_vehicle_gravity_physics() {
    let mut app = create_test_app();
    let (vehicle_entity, _, _) = setup_test_scene(&mut app);
    
    // Run simulation for 1 second to let vehicle settle
    run_simulation_duration(&mut app, 1.0);
    
    // Vehicle should have settled due to gravity
    let world = app.world();
    let transform = world.get::<Transform>(vehicle_entity).unwrap();
    let velocity = world.get::<Velocity>(vehicle_entity).unwrap();
    
    // Vehicle should have settled due to gravity
    assert!(transform.translation.y < 1.0, "Vehicle should settle on ground");
    assert!(velocity.linvel.length() < 0.1, "Vehicle should be at rest");
}

#[test]
fn test_vehicle_acceleration_physics() {
    let mut app = create_test_app();
    let (vehicle_entity, _, _) = setup_test_scene(&mut app);
    
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
    let world = app.world();
    let final_velocity = world.get::<Velocity>(vehicle_entity).unwrap().linvel.length();
    assert!(final_velocity > initial_velocity, "Vehicle should accelerate with input");
    assert!(final_velocity < 50.0, "Vehicle should not exceed reasonable speed");
}

#[test]
fn test_vehicle_braking_physics() {
    let mut app = create_test_app();
    let (vehicle_entity, _, _) = setup_test_scene(&mut app);
    
    // First, give vehicle some initial velocity
    {
        let mut world = app.world_mut();
        if let Some(mut velocity) = world.get_mut::<Velocity>(vehicle_entity) {
            velocity.linvel = Vec3::new(0.0, 0.0, 20.0); // 20 m/s forward
        }
    }
    
    // Set up control manager with braking input
    let mut control_manager = ControlManager::new();
    control_manager.set_control_value(ControlAction::Brake, 1.0);
    app.insert_resource(control_manager);
    
    let initial_speed = {
        let world = app.world();
        world.get::<Velocity>(vehicle_entity).unwrap().linvel.length()
    };
    
    // Run simulation
    run_simulation_duration(&mut app, 1.0);
    
    // Check that vehicle decelerated
    let world = app.world();
    let final_speed = world.get::<Velocity>(vehicle_entity).unwrap().linvel.length();
    assert!(final_speed < initial_speed, "Vehicle should decelerate with braking");
}

#[test]
fn test_vehicle_steering_physics() {
    let mut app = create_test_app();
    let (vehicle_entity, _, _) = setup_test_scene(&mut app);
    
    // Give vehicle forward velocity and steering input
    {
        let mut world = app.world_mut();
        if let Some(mut velocity) = world.get_mut::<Velocity>(vehicle_entity) {
            velocity.linvel = Vec3::new(0.0, 0.0, 10.0); // 10 m/s forward
        }
    }
    
    let mut control_manager = ControlManager::new();
    control_manager.set_control_value(ControlAction::Steer, 0.5); // Turn right
    app.insert_resource(control_manager);
    
    let initial_position = {
        let world = app.world();
        world.get::<Transform>(vehicle_entity).unwrap().translation
    };
    
    // Run simulation
    run_simulation_duration(&mut app, 1.0);
    
    // Check that vehicle turned (x position should change)
    let world = app.world();
    let final_position = world.get::<Transform>(vehicle_entity).unwrap().translation;
    assert!(
        (final_position.x - initial_position.x).abs() > 0.5,
        "Vehicle should turn when steering input is applied"
    );
}

#[test]
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
    };
    
    let heavy_vehicle = {
        let mut world = app2.world_mut();
        world.spawn((
            Car::default(),
            ActiveEntity,
            Transform::from_xyz(0.0, 1.0, 0.0),
            GlobalTransform::default(),
            Velocity::default(),
            RigidBody::Dynamic,
            Collider::cuboid(2.0, 1.0, 4.0),
            AdditionalMassProperties::Mass(3000.0), // Heavy vehicle
            RealisticVehicle::default(),
            VehicleDynamics::default(),
            EnginePhysics::default(),
            VehicleSuspension::default(),
            TirePhysics::default(),
        )).id()
    };
    
    // Apply same acceleration to both
    let mut control_manager = ControlManager::new();
    control_manager.set_control_value(ControlAction::Accelerate, 1.0);
    app1.insert_resource(control_manager.clone());
    app2.insert_resource(control_manager);
    
    // Run simulation
    run_simulation_duration(&mut app1, 2.0);
    run_simulation_duration(&mut app2, 2.0);
    
    // Check that lighter vehicle accelerates more
    let light_speed = {
        let world = app1.world();
        world.get::<Velocity>(light_vehicle).unwrap().linvel.length()
    };
    let heavy_speed = {
        let world = app2.world();
        world.get::<Velocity>(heavy_vehicle).unwrap().linvel.length()
    };
    
    assert!(
        light_speed > heavy_speed,
        "Lighter vehicle should accelerate faster than heavier vehicle"
    );
}
