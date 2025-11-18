#![allow(dead_code)]
/// Test entity spawning utilities
/// Provides simplified vehicle creation for testing without full game dependencies
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use gta_game::components::{Car, ControlState, F16, Helicopter, VehicleState, VehicleType};

/// Spawn a basic test car with minimal components
/// Returns entity ID for querying in tests
///
/// # Physics Configuration
/// - RigidBody::Dynamic with simplified collider
/// - Mass: 1200kg, Damping: linear 0.1, angular 3.5
/// - CCD enabled for continuous collision detection
///
/// # Example
/// ```ignore
/// let mut app = setup_test_app_with_rapier();
/// let car = spawn_test_car(&mut app, Vec3::new(0.0, 5.0, 0.0));
/// run_app_updates(&mut app, 10);
/// ```
pub fn spawn_test_car(app: &mut App, position: Vec3) -> Entity {
    let entity = app
        .world_mut()
        .spawn((
            Name::new("TestCar"),
            Car,
            VehicleState::new(VehicleType::SuperCar),
            ControlState::default(),
            Transform::from_translation(position),
            Visibility::default(),
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
            RigidBody::Dynamic,
            Collider::cuboid(0.72, 0.5, 1.68), // Simplified car collider
            Velocity::default(),
            AdditionalMassProperties::Mass(1200.0),
            Damping {
                linear_damping: 0.1,
                angular_damping: 3.5,
            },
            Ccd::enabled(),
            CollisionGroups::new(Group::GROUP_2, Group::ALL),
        ))
        .id();
    entity
}

/// Spawn a basic test helicopter with minimal components
/// Returns entity ID for querying in tests
///
/// # Physics Configuration
/// - RigidBody::Dynamic with capsule collider
/// - Mass: 2000kg, Damping: linear 0.3, angular 1.5
/// - Higher damping for stable flight testing
///
/// # Example
/// ```ignore
/// let mut app = setup_test_app_with_rapier();
/// let heli = spawn_test_helicopter(&mut app, Vec3::new(0.0, 20.0, 0.0));
/// ```
pub fn spawn_test_helicopter(app: &mut App, position: Vec3) -> Entity {
    let entity = app
        .world_mut()
        .spawn((
            Name::new("TestHelicopter"),
            Helicopter,
            VehicleState::new(VehicleType::Helicopter),
            ControlState::default(),
            Transform::from_translation(position),
            Visibility::default(),
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
            RigidBody::Dynamic,
            Collider::capsule_y(2.0, 1.2), // Simplified helicopter collider (half-height 2.0, radius 1.2)
            Velocity::default(),
            AdditionalMassProperties::Mass(2000.0),
            Damping {
                linear_damping: 0.3,
                angular_damping: 1.5,
            },
            Ccd::enabled(),
            CollisionGroups::new(Group::GROUP_2, Group::ALL),
        ))
        .id();
    entity
}

/// Spawn a basic test F16 with minimal components
/// Returns entity ID for querying in tests
///
/// # Physics Configuration
/// - RigidBody::Dynamic with capsule collider
/// - Mass: 12000kg (combat loaded), Damping: linear 0.05, angular 0.5
/// - Low damping for high-speed flight testing
///
/// # Example
/// ```ignore
/// let mut app = setup_test_app_with_rapier();
/// let f16 = spawn_test_f16(&mut app, Vec3::new(0.0, 100.0, 0.0));
/// ```
pub fn spawn_test_f16(app: &mut App, position: Vec3) -> Entity {
    let entity = app
        .world_mut()
        .spawn((
            Name::new("TestF16"),
            F16,
            VehicleState::new(VehicleType::F16),
            ControlState::default(),
            Transform::from_translation(position),
            Visibility::default(),
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
            RigidBody::Dynamic,
            Collider::capsule_z(4.0, 6.0), // Simplified F16 collider (half-height 4.0, radius 6.0, Z-axis aligned)
            Velocity::default(),
            AdditionalMassProperties::Mass(12000.0),
            Damping {
                linear_damping: 0.05,
                angular_damping: 0.5,
            },
            Ccd::enabled(),
            CollisionGroups::new(Group::GROUP_2, Group::ALL),
        ))
        .id();
    entity
}

/// Get current velocity from entity
/// Returns Vec3::ZERO if entity doesn't exist or has no Velocity component
pub fn get_test_velocity(app: &App, entity: Entity) -> Vec3 {
    app.world()
        .get::<Velocity>(entity)
        .map(|v| v.linvel)
        .unwrap_or(Vec3::ZERO)
}

/// Set control state for entity (throttle, brake, steering)
/// Useful for simulating player input in tests
///
/// # Arguments
/// * `app` - Mutable reference to the Bevy App
/// * `entity` - Entity to modify
/// * `throttle` - Forward acceleration (0.0 to 1.0)
/// * `brake` - Braking force (0.0 to 1.0)
/// * `steering` - Steering input (-1.0 left, 0.0 straight, 1.0 right)
///
/// # Example
/// ```ignore
/// set_test_control_state(&mut app, car, 1.0, 0.0, 0.5); // Full throttle, no brake, slight right turn
/// ```
pub fn set_test_control_state(
    app: &mut App,
    entity: Entity,
    throttle: f32,
    brake: f32,
    steering: f32,
) {
    if let Some(mut control_state) = app.world_mut().get_mut::<ControlState>(entity) {
        control_state.throttle = throttle;
        control_state.brake = brake;
        control_state.steering = steering;
    }
}

/// Set vertical control for aircraft (helicopter/F16)
/// Useful for testing climb/descent behavior
///
/// # Arguments
/// * `app` - Mutable reference to the Bevy App
/// * `entity` - Entity to modify
/// * `vertical` - Vertical input (-1.0 down, 0.0 neutral, 1.0 up)
pub fn set_test_vertical_control(app: &mut App, entity: Entity, vertical: f32) {
    if let Some(mut control_state) = app.world_mut().get_mut::<ControlState>(entity) {
        control_state.vertical = vertical;
    }
}

/// Set pitch/yaw/roll for aircraft testing
pub fn set_test_flight_controls(app: &mut App, entity: Entity, pitch: f32, yaw: f32, roll: f32) {
    if let Some(mut control_state) = app.world_mut().get_mut::<ControlState>(entity) {
        control_state.pitch = pitch;
        control_state.yaw = yaw;
        control_state.roll = roll;
    }
}

/// Apply immediate velocity to entity (for testing physics directly)
/// Bypasses normal movement systems
pub fn apply_test_velocity(app: &mut App, entity: Entity, velocity: Vec3) {
    if let Some(mut vel) = app.world_mut().get_mut::<Velocity>(entity) {
        vel.linvel = velocity;
    }
}

/// Spawn ground plane for collision testing
/// Returns entity ID of the static ground
pub fn spawn_test_ground(app: &mut App, position: Vec3, size: f32) -> Entity {
    let entity = app
        .world_mut()
        .spawn((
            Name::new("TestGround"),
            Transform::from_translation(position),
            Visibility::default(),
            RigidBody::Fixed,
            Collider::cuboid(size, 0.1, size), // Thin ground plane
            CollisionGroups::new(Group::GROUP_1, Group::ALL),
        ))
        .id();
    entity
}
