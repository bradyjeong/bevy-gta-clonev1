use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::factories::BundleFactory;

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
        BundleFactory::create_vehicle_physics_bundle(Vec3::new(20.0, 0.5, 12.0)),
        BundleFactory::create_basic_car_collision(),
        BundleFactory::create_standard_vehicle_locked_axes(),
        BundleFactory::create_standard_vehicle_damping(),
        GlobalTransform::default(),
        Visibility::default(),
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
        BundleFactory::create_vehicle_physics_bundle(Vec3::new(5.0, 1.3, 0.0)),
        BundleFactory::create_super_car_collision(),
        BundleFactory::create_standard_vehicle_locked_axes(),
        BundleFactory::create_standard_vehicle_damping(),
        GlobalTransform::default(),
        Visibility::default(),
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
        BundleFactory::create_vehicle_physics_bundle(Vec3::new(120.0, 15.0, 80.0)),
        BundleFactory::create_helicopter_collision(),
        GlobalTransform::default(),
        Visibility::default(),
        Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI)), // Additional rotation
        Damping { linear_damping: 2.0, angular_damping: 8.0 }, // Override default damping
        LockedAxes::empty(), // Helicopter has full 6DOF movement
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
        BundleFactory::create_vehicle_physics_bundle(Vec3::new(80.0, 2.0, 120.0)),
        BundleFactory::create_f16_collision(),
        GlobalTransform::default(),
        Visibility::default(),
        LockedAxes::empty(), // F16 has full 6DOF movement for realistic flight
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
    
    let collider_bundle = BundleFactory::create_vehicle_collision_bundle(vehicle_type);
    
    let locked_axes = match vehicle_type {
        VehicleType::BasicCar | VehicleType::SuperCar => 
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        VehicleType::Helicopter | VehicleType::F16 => LockedAxes::empty(),
    };
    
    commands.spawn((
        vehicle_state,
        BundleFactory::create_vehicle_physics_bundle(position),
        collider_bundle,
        BundleFactory::create_standard_vehicle_damping(),
        GlobalTransform::default(),
        Visibility::default(),
        Transform::from_rotation(rotation), // Additional rotation override
        locked_axes, // Locked axes based on vehicle type
    )).id()
}
