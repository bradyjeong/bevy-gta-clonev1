use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::constants::*;

/// Simplified vehicle setup - no deprecated factories
pub fn setup_simple_vehicles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Basic car
    let car_entity = commands.spawn((
        Car,
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 0.5, 2.0),  // Half-height = 0.5, total height = 1.0
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        Velocity::zero(),
        Transform::from_xyz(15.0, 0.5, 0.0),  // Fixed: spawn at proper ground height
        Damping { linear_damping: 1.0, angular_damping: 5.0 },
        Cullable { max_distance: 300.0, is_culled: false },
        CollisionGroups::new(VEHICLE_GROUP, STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP),
    )).id();

    // Car body - Fixed: height matches collider
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.8, 1.0, 3.6))),  // Fixed: height 1.0 matches collider
        MeshMaterial3d(materials.add(Color::srgb(1.0, 0.0, 0.0))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ChildOf(car_entity),
    ));

    // Supercar
    let supercar_entity = commands.spawn((
        Car,
        SuperCar {
            max_speed: 120.0,
            acceleration: 40.0,
            turbo_boost: false,
            exhaust_timer: 0.0,
        },
        RigidBody::Dynamic,
        Collider::cuboid(1.1, 0.5, 2.4),  // Half-height = 0.5, total height = 1.0
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        Velocity::zero(),
        Transform::from_xyz(3.0, 0.5, 0.0),  // Fixed: spawn at proper ground height
        Damping { linear_damping: 1.0, angular_damping: 5.0 },
        Cullable { max_distance: 800.0, is_culled: false },
        CollisionGroups::new(VEHICLE_GROUP, STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP),
    )).id();

    // Supercar body - Fixed: height matches collider
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 1.0, 4.5))),  // Fixed: height 1.0 matches collider
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.05, 0.05, 0.15),
            metallic: 0.9,
            reflectance: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ChildOf(supercar_entity),
    ));
}

/// Simplified helicopter setup
pub fn setup_simple_helicopter(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let helicopter_entity = commands.spawn((
        Helicopter,
        RigidBody::Dynamic,
        Collider::cuboid(1.5, 1.0, 3.0),  // Half-height = 1.0, total height = 2.0
        Velocity::zero(),
        Transform::from_xyz(120.0, 1.0, 80.0),  // Fixed: spawn at ground+half-height (hovering)
        Damping { linear_damping: 2.0, angular_damping: 8.0 },
        CollisionGroups::new(VEHICLE_GROUP, STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP),
    )).id();

    // Helicopter body - Fixed: dimensions match collider
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(3.0, 2.0, 6.0))),  // Fixed: dimensions match collider
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.9, 0.9))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ChildOf(helicopter_entity),
    ));
}

/// Simplified F16 setup
pub fn setup_simple_f16(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let f16_entity = commands.spawn((
        F16,
        RigidBody::Dynamic,
        Collider::cuboid(8.0, 1.5, 1.5),  // Half-height = 1.5, total height = 3.0
        LockedAxes::empty(),
        Velocity::zero(),
        Transform::from_xyz(80.0, 1.5, 120.0),  // Fixed: spawn at ground+half-height
        Cullable { max_distance: 2000.0, is_culled: false },
    )).id();

    // F16 body - Fixed: dimensions match collider
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(16.0, 3.0, 3.0))),  // Fixed: height 3.0 matches collider
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.4, 0.5),
            metallic: 0.8,
            perceptual_roughness: 0.2,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ChildOf(f16_entity),
    ));
}
