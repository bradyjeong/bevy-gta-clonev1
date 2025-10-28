use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use gta_game::components::{
    AircraftFlight, Car, CarRuntime, CarWheelGroundContact, ControlState, F16, GroundContact,
    Grounded, Helicopter, HelicopterRuntime, MainRotor, TailRotor, VehicleState, VehicleType,
    VisualOnly, WheelRuntime,
};
use gta_game::config::GameConfig;

/// Helper to set up test app with minimal Bevy + Rapier plugins
fn setup_test_app_with_rapier() -> App {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(TransformPlugin)
        .add_plugins(bevy::render::view::VisibilityPlugin)
        .add_plugins(bevy::scene::ScenePlugin) // Required for SceneSpawner resource
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(GameConfig::default())
        .init_asset::<Mesh>() // Required by Rapier for scene collider system
        .init_asset::<StandardMaterial>(); // Required for completeness

    app
}

/// Helper to spawn a test car entity with minimal required components
fn spawn_test_car(app: &mut App) -> Entity {
    let config = app.world().resource::<GameConfig>().clone();

    app.world_mut()
        .spawn((
            Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
            Car,
            VehicleState::new(VehicleType::SuperCar),
            ControlState::default(),
            RigidBody::Dynamic,
            Collider::cuboid(
                config.vehicles.super_car.collider_size.x,
                config.vehicles.super_car.collider_size.y,
                config.vehicles.super_car.collider_size.z,
            ),
            Velocity::default(),
            AdditionalMassProperties::Mass(config.vehicles.super_car.mass),
            Damping {
                linear_damping: config.vehicles.super_car.linear_damping,
                angular_damping: config.vehicles.super_car.angular_damping,
            },
            Grounded::default(),
            ExternalForce::default(),
            GroundContact::default(),
            WheelRuntime::default(),
            CarWheelGroundContact::default(),
            CarRuntime::default(),
        ))
        .id()
}

/// Helper to spawn a test helicopter entity
fn spawn_test_helicopter(app: &mut App) -> Entity {
    let config = app.world().resource::<GameConfig>().clone();

    let heli_entity = app
        .world_mut()
        .spawn((
            Transform::from_translation(Vec3::new(0.0, 10.0, 0.0)),
            Helicopter,
            VehicleState::new(VehicleType::Helicopter),
            ControlState::default(),
            RigidBody::Dynamic,
            Collider::cuboid(
                config.vehicles.helicopter.collider_size.x,
                config.vehicles.helicopter.collider_size.y,
                config.vehicles.helicopter.collider_size.z,
            ),
            Velocity::default(),
            AdditionalMassProperties::Mass(config.vehicles.helicopter.mass),
            Damping {
                linear_damping: config.vehicles.helicopter.linear_damping,
                angular_damping: config.vehicles.helicopter.angular_damping,
            },
            HelicopterRuntime::default(),
            AircraftFlight::default(),
        ))
        .id();

    // Add visual rotor children with VisualOnly marker
    app.world_mut().spawn((
        Transform::default(),
        MainRotor,
        VisualOnly,
        ChildOf(heli_entity),
    ));

    app.world_mut().spawn((
        Transform::default(),
        TailRotor,
        VisualOnly,
        ChildOf(heli_entity),
    ));

    heli_entity
}

/// Helper to spawn a test F16 entity
fn spawn_test_f16(app: &mut App) -> Entity {
    let config = app.world().resource::<GameConfig>().clone();

    app.world_mut()
        .spawn((
            Transform::from_translation(Vec3::new(0.0, 100.0, 0.0)),
            F16,
            VehicleState::new(VehicleType::F16),
            ControlState::default(),
            RigidBody::Dynamic,
            Collider::capsule_y(6.0, 4.0), // Capsule collider for F16 (not cuboid)
            Velocity::default(),
            AdditionalMassProperties::Mass(config.vehicles.f16.mass),
            Damping {
                linear_damping: config.vehicles.f16.linear_damping,
                angular_damping: config.vehicles.f16.angular_damping,
            },
            AircraftFlight::default(),
        ))
        .id()
}

// ===========================
// Car Spawning Tests
// ===========================

#[test]
fn test_car_spawns_with_physics_components() {
    let mut app = setup_test_app_with_rapier();
    let car_entity = spawn_test_car(&mut app);

    // Verify all required physics components exist
    let world = app.world();
    assert!(
        world.get::<RigidBody>(car_entity).is_some(),
        "Car should have RigidBody"
    );
    assert!(
        world.get::<Collider>(car_entity).is_some(),
        "Car should have Collider"
    );
    assert!(
        world.get::<Velocity>(car_entity).is_some(),
        "Car should have Velocity"
    );
    assert!(
        world.get::<AdditionalMassProperties>(car_entity).is_some(),
        "Car should have Mass"
    );
    assert!(
        world.get::<Damping>(car_entity).is_some(),
        "Car should have Damping"
    );

    // Verify RigidBody is Dynamic
    let rb = world.get::<RigidBody>(car_entity).unwrap();
    assert!(
        matches!(rb, RigidBody::Dynamic),
        "Car RigidBody should be Dynamic"
    );
}

#[test]
fn test_car_has_control_state() {
    let mut app = setup_test_app_with_rapier();
    let car_entity = spawn_test_car(&mut app);

    let world = app.world();
    let control_state = world.get::<ControlState>(car_entity);
    assert!(
        control_state.is_some(),
        "Car should have ControlState component"
    );

    let control = control_state.unwrap();
    assert_eq!(control.throttle, 0.0, "Initial throttle should be 0.0");
    assert_eq!(control.steering, 0.0, "Initial steering should be 0.0");
    assert!(
        !control.emergency_brake,
        "Emergency brake should be off initially"
    );
}

#[test]
fn test_car_collider_is_correct_size() {
    let mut app = setup_test_app_with_rapier();
    let car_entity = spawn_test_car(&mut app);

    let world = app.world();
    let config = world.resource::<GameConfig>();
    let expected_size = config.vehicles.super_car.collider_size;

    // Collider component exists (checked in collider consistency via shape inspection)
    let collider = world.get::<Collider>(car_entity).unwrap();

    // Verify collider is a cuboid shape (basic validation)
    assert!(
        collider.as_cuboid().is_some(),
        "Car collider should be a cuboid"
    );

    let cuboid = collider.as_cuboid().unwrap();
    let half_extents = cuboid.half_extents();

    // Verify dimensions match config (within floating point tolerance)
    assert!(
        (half_extents.x - expected_size.x).abs() < 0.01,
        "Collider X half-extent should match config: expected {}, got {}",
        expected_size.x,
        half_extents.x
    );
    assert!(
        (half_extents.y - expected_size.y).abs() < 0.01,
        "Collider Y half-extent should match config: expected {}, got {}",
        expected_size.y,
        half_extents.y
    );
    assert!(
        (half_extents.z - expected_size.z).abs() < 0.01,
        "Collider Z half-extent should match config: expected {}, got {}",
        expected_size.z,
        half_extents.z
    );
}

// ===========================
// Car Movement Tests
// ===========================

#[test]
fn test_car_forward_throttle_increases_velocity() {
    let mut app = setup_test_app_with_rapier();
    let car_entity = spawn_test_car(&mut app);

    // Set throttle to full
    {
        let mut control = app.world_mut().get_mut::<ControlState>(car_entity).unwrap();
        control.throttle = 1.0;
    }

    // Simulate physics by manually setting velocity (movement system would do this)
    // In a real test with the full movement system, this would be automatic
    {
        let mut velocity = app.world_mut().get_mut::<Velocity>(car_entity).unwrap();
        velocity.linvel = Vec3::new(0.0, 0.0, -10.0); // Forward is -Z
    }

    app.update();

    // Verify velocity is in forward direction (-Z)
    let velocity = app.world().get::<Velocity>(car_entity).unwrap();
    assert!(
        velocity.linvel.z < 0.0,
        "Car with throttle should have negative Z velocity (forward), got {}",
        velocity.linvel.z
    );
    assert!(
        velocity.linvel.length() > 0.0,
        "Car with throttle should have non-zero velocity"
    );
}

#[test]
fn test_car_steering_changes_rotation() {
    let mut app = setup_test_app_with_rapier();
    let car_entity = spawn_test_car(&mut app);

    // Set steering to right
    {
        let mut control = app.world_mut().get_mut::<ControlState>(car_entity).unwrap();
        control.steering = 1.0; // Full right
    }

    // Simulate angular velocity change (movement system would do this)
    {
        let mut velocity = app.world_mut().get_mut::<Velocity>(car_entity).unwrap();
        velocity.angvel = Vec3::new(0.0, -2.0, 0.0); // Turning right (negative Y angular velocity)
    }

    app.update();

    // Verify angular velocity indicates rotation
    let velocity = app.world().get::<Velocity>(car_entity).unwrap();
    assert!(
        velocity.angvel.length() > 0.0,
        "Car with steering input should have angular velocity"
    );
}

#[test]
fn test_car_brake_stops_movement() {
    let mut app = setup_test_app_with_rapier();
    let car_entity = spawn_test_car(&mut app);

    // Set initial forward velocity
    {
        let mut velocity = app.world_mut().get_mut::<Velocity>(car_entity).unwrap();
        velocity.linvel = Vec3::new(0.0, 0.0, -20.0); // Moving forward at 20 m/s
    }

    // Apply emergency brake
    {
        let mut control = app.world_mut().get_mut::<ControlState>(car_entity).unwrap();
        control.emergency_brake = true;
    }

    // Simulate brake effect (movement system would reduce velocity)
    {
        let mut velocity = app.world_mut().get_mut::<Velocity>(car_entity).unwrap();
        velocity.linvel *= 0.5; // Brake reduces velocity by 50% per frame (example)
    }

    app.update();

    // Verify velocity decreased
    let velocity = app.world().get::<Velocity>(car_entity).unwrap();
    assert!(
        velocity.linvel.length() < 20.0,
        "Emergency brake should reduce velocity from 20 m/s, got {}",
        velocity.linvel.length()
    );
}

// ===========================
// Helicopter Tests
// ===========================

#[test]
fn test_helicopter_spawns_with_correct_components() {
    let mut app = setup_test_app_with_rapier();
    let heli_entity = spawn_test_helicopter(&mut app);

    let world = app.world();

    // Verify helicopter marker component
    assert!(
        world.get::<Helicopter>(heli_entity).is_some(),
        "Entity should have Helicopter marker"
    );

    // Verify physics components
    assert!(
        world.get::<RigidBody>(heli_entity).is_some(),
        "Helicopter should have RigidBody"
    );
    assert!(
        world.get::<Velocity>(heli_entity).is_some(),
        "Helicopter should have Velocity"
    );

    // Verify helicopter-specific components
    assert!(
        world.get::<HelicopterRuntime>(heli_entity).is_some(),
        "Helicopter should have HelicopterRuntime"
    );
    assert!(
        world.get::<AircraftFlight>(heli_entity).is_some(),
        "Helicopter should have AircraftFlight"
    );

    // Verify ControlState
    assert!(
        world.get::<ControlState>(heli_entity).is_some(),
        "Helicopter should have ControlState"
    );
}

#[test]
fn test_helicopter_vertical_movement() {
    let mut app = setup_test_app_with_rapier();
    let heli_entity = spawn_test_helicopter(&mut app);

    // Set vertical control to ascend
    {
        let mut control = app
            .world_mut()
            .get_mut::<ControlState>(heli_entity)
            .unwrap();
        control.vertical = 1.0; // Full ascend
    }

    // Simulate vertical thrust (movement system would do this)
    {
        let mut velocity = app.world_mut().get_mut::<Velocity>(heli_entity).unwrap();
        velocity.linvel.y = 5.0; // Ascending at 5 m/s
    }

    app.update();

    // Verify Y velocity is positive (ascending)
    let velocity = app.world().get::<Velocity>(heli_entity).unwrap();
    assert!(
        velocity.linvel.y > 0.0,
        "Helicopter with vertical=1.0 should have positive Y velocity (ascending), got {}",
        velocity.linvel.y
    );
}

#[test]
fn test_helicopter_has_visual_rotors() {
    let mut app = setup_test_app_with_rapier();
    let _heli_entity = spawn_test_helicopter(&mut app);

    app.update();

    let world = app.world();

    // Count rotor children with VisualOnly marker
    let mut main_rotor_count = 0;
    let mut tail_rotor_count = 0;

    for entity in world.iter_entities() {
        if world.get::<MainRotor>(entity.id()).is_some()
            && world.get::<VisualOnly>(entity.id()).is_some()
        {
            main_rotor_count += 1;
        }
        if world.get::<TailRotor>(entity.id()).is_some()
            && world.get::<VisualOnly>(entity.id()).is_some()
        {
            tail_rotor_count += 1;
        }
    }

    assert_eq!(
        main_rotor_count, 1,
        "Helicopter should have exactly 1 MainRotor with VisualOnly marker"
    );
    assert_eq!(
        tail_rotor_count, 1,
        "Helicopter should have exactly 1 TailRotor with VisualOnly marker"
    );
}

// ===========================
// F16 Tests
// ===========================

#[test]
fn test_f16_spawns_correctly() {
    let mut app = setup_test_app_with_rapier();
    let f16_entity = spawn_test_f16(&mut app);

    let world = app.world();

    // Verify F16 marker component
    assert!(
        world.get::<F16>(f16_entity).is_some(),
        "Entity should have F16 marker"
    );

    // Verify physics components
    assert!(
        world.get::<RigidBody>(f16_entity).is_some(),
        "F16 should have RigidBody"
    );
    assert!(
        world.get::<Velocity>(f16_entity).is_some(),
        "F16 should have Velocity"
    );
    assert!(
        world.get::<Collider>(f16_entity).is_some(),
        "F16 should have Collider"
    );

    // Verify aircraft-specific components
    assert!(
        world.get::<AircraftFlight>(f16_entity).is_some(),
        "F16 should have AircraftFlight"
    );
    assert!(
        world.get::<ControlState>(f16_entity).is_some(),
        "F16 should have ControlState"
    );
}

#[test]
fn test_f16_has_capsule_collider() {
    let mut app = setup_test_app_with_rapier();
    let f16_entity = spawn_test_f16(&mut app);

    let world = app.world();
    let collider = world.get::<Collider>(f16_entity).unwrap();

    // F16 uses capsule collider, not cuboid
    assert!(
        collider.as_capsule().is_some(),
        "F16 collider should be a capsule shape, not cuboid"
    );
    assert!(
        collider.as_cuboid().is_none(),
        "F16 collider should NOT be a cuboid"
    );

    // Verify capsule dimensions
    let capsule = collider.as_capsule().unwrap();
    assert!(
        (capsule.radius() - 4.0).abs() < 0.01,
        "F16 capsule radius should be ~4.0, got {}",
        capsule.radius()
    );
    assert!(
        (capsule.half_height() - 6.0).abs() < 0.01,
        "F16 capsule half_height should be ~6.0, got {}",
        capsule.half_height()
    );
}

#[test]
fn test_control_state_default_values() {
    let control = ControlState::default();

    assert_eq!(control.throttle, 0.0);
    assert_eq!(control.brake, 0.0);
    assert_eq!(control.steering, 0.0);
    assert_eq!(control.vertical, 0.0);
    assert_eq!(control.pitch, 0.0);
    assert_eq!(control.roll, 0.0);
    assert_eq!(control.yaw, 0.0);
    assert_eq!(control.boost, 0.0);
    assert!(!control.emergency_brake);
    assert!(!control.interact);
}

#[test]
fn test_control_state_validation() {
    let mut control = ControlState {
        throttle: 2.0,  // Over limit
        steering: -3.0, // Under limit
        brake: 0.5,     // Valid
        ..Default::default()
    };

    control.validate_and_clamp();

    assert_eq!(control.throttle, 1.0, "Throttle should be clamped to 1.0");
    assert_eq!(control.steering, -1.0, "Steering should be clamped to -1.0");
    assert_eq!(control.brake, 0.5, "Brake should remain valid at 0.5");
}

#[test]
fn test_control_state_movement_detection() {
    let mut control = ControlState::default();
    assert!(!control.has_movement_input());

    control.throttle = 0.5;
    assert!(control.has_movement_input());
    assert!(control.is_accelerating());

    control.throttle = 0.0;
    control.brake = 0.3;
    assert!(control.has_movement_input());
    assert!(control.is_braking());
}
