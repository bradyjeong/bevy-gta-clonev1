use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::constants::*;

// NEW LOD-BASED VEHICLE SPAWNING

pub fn setup_lod_vehicles(
    mut commands: Commands,
) {
    // Spawn vehicles with only state components initially
    // LOD system will add/remove rendering as needed based on distance
    
    // Basic car for testing
    commands.spawn((
        VehicleState::new(VehicleType::BasicCar),
        Car, // Keep legacy marker for compatibility
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 0.5, 2.0),
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        Velocity::zero(),
        Transform::from_xyz(15.0, 0.5, 8.0),
        GlobalTransform::default(),
        Visibility::default(),
        CollisionGroups::new(VEHICLE_GROUP, STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP),
        Damping { linear_damping: 1.0, angular_damping: 5.0 },
    ));

    // Bugatti Chiron SuperCar
    commands.spawn((
        VehicleState {
            vehicle_type: VehicleType::SuperCar,
            color: Color::srgb(0.05, 0.05, 0.15), // Dark blue
            max_speed: 120.0,
            acceleration: 40.0,
            damage: 0.0,
            fuel: 100.0,
            current_lod: VehicleLOD::StateOnly,
            last_lod_check: 0.0,
        },
        SuperCar {
            max_speed: 120.0,
            acceleration: 40.0,
            turbo_boost: false,
            exhaust_timer: 0.0,
        },
        Transform::from_xyz(5.0, 1.3, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 0.5, 2.0),
        Velocity::zero(),
        Friction::coefficient(0.3),
        Restitution::coefficient(0.0),
        Ccd::enabled(),
        CollisionGroups::new(VEHICLE_GROUP, STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP),
        Damping { linear_damping: 1.0, angular_damping: 5.0 },
    ));

    // Add beacon for Bugatti Chiron (still static for now)
    commands.spawn((
        Transform::from_xyz(5.0, 4.0, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        VehicleBeacon,
    ));
}

pub fn setup_lod_helicopter(
    mut commands: Commands,
) {
    // HELICOPTER - Spawn with LOD system
    commands.spawn((
        VehicleState {
            vehicle_type: VehicleType::Helicopter,
            color: Color::srgb(0.9, 0.9, 0.9),
            max_speed: 80.0,
            acceleration: 25.0,
            damage: 0.0,
            fuel: 100.0,
            current_lod: VehicleLOD::StateOnly,
            last_lod_check: 0.0,
        },
        Helicopter,
        RigidBody::Dynamic,
        Collider::cuboid(1.5, 1.0, 3.0),
        Velocity::zero(),
        Transform::from_xyz(120.0, 15.0, 80.0).with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        GlobalTransform::default(),
        Visibility::default(),
        Ccd::enabled(),
        CollisionGroups::new(VEHICLE_GROUP, STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP),
        Damping { linear_damping: 2.0, angular_damping: 8.0 },
    ));

    // Add beacon for Helicopter
    commands.spawn((
        Transform::from_xyz(120.0, 23.0, 80.0),
        GlobalTransform::default(),
        Visibility::default(),
        VehicleBeacon,
    ));
}

pub fn setup_lod_f16(
    mut commands: Commands,
) {
    // F16 FIGHTER JET - Spawn with LOD system
    commands.spawn((
        VehicleState {
            vehicle_type: VehicleType::F16,
            color: Color::srgb(0.4, 0.4, 0.5),
            max_speed: 300.0,
            acceleration: 100.0,
            damage: 0.0,
            fuel: 100.0,
            current_lod: VehicleLOD::StateOnly,
            last_lod_check: 0.0,
        },
        F16,
        RigidBody::Dynamic,
        Collider::cuboid(8.0, 1.5, 1.5),
        LockedAxes::empty(), // Full 6DOF movement for realistic flight
        Velocity::zero(),
        Transform::from_xyz(80.0, 2.0, 120.0),
        GlobalTransform::default(),
        Visibility::default(),
    ));

    // Add beacon for F16
    commands.spawn((
        Transform::from_xyz(80.0, 10.0, 120.0),
        GlobalTransform::default(),
        Visibility::default(),
        VehicleBeacon,
    ));
}

// Utility function to spawn a vehicle with LOD system
pub fn spawn_vehicle_with_lod(
    commands: &mut Commands,
    vehicle_type: VehicleType,
    position: Vec3,
    rotation: Quat,
) -> Entity {
    let vehicle_state = VehicleState::new(vehicle_type);
    
    let collider_size = match vehicle_type {
        VehicleType::BasicCar | VehicleType::SuperCar => Collider::cuboid(1.0, 0.5, 2.0),
        VehicleType::Helicopter => Collider::cuboid(1.5, 1.0, 3.0),
        VehicleType::F16 => Collider::cuboid(8.0, 1.5, 1.5),
    };
    
    let locked_axes = match vehicle_type {
        VehicleType::BasicCar | VehicleType::SuperCar => 
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        VehicleType::Helicopter | VehicleType::F16 => LockedAxes::empty(),
    };
    
    commands.spawn((
        vehicle_state,
        RigidBody::Dynamic,
        collider_size,
        locked_axes,
        Velocity::zero(),
        Transform::from_translation(position).with_rotation(rotation),
        GlobalTransform::default(),
        Visibility::default(),
        CollisionGroups::new(VEHICLE_GROUP, STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP),
        Damping { linear_damping: 1.0, angular_damping: 5.0 },
    )).id()
}
