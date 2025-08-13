use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::bundles::VehicleVisibilityBundle;

pub fn setup_starter_vehicles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Add a few starter vehicles in non-overlapping positions
    let starter_positions = [
        Vec3::new(25.0, 0.5, 10.0),   // Near spawn, well spaced
        Vec3::new(-30.0, 0.5, 15.0),  // Different area
        Vec3::new(10.0, 0.5, -25.0),  // Another area
    ];
    
    let car_colors = [
        Color::srgb(1.0, 0.0, 0.0), // Red
        Color::srgb(0.0, 0.0, 1.0), // Blue  
        Color::srgb(0.0, 1.0, 0.0), // Green
    ];
    
    for (i, position) in starter_positions.iter().enumerate() {
        let color = car_colors[i % car_colors.len()];
        
        // Create car parent entity with physics
        let car_entity = commands.spawn((
            Car,
            RigidBody::Dynamic,
            Collider::cuboid(1.0, 0.5, 2.0),
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
            Velocity::zero(),
            Transform::from_xyz(position.x, position.y, position.z),
            VehicleVisibilityBundle::default(),
            CollisionGroups::new(VEHICLE_GROUP, STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP),
            Damping { linear_damping: 1.0, angular_damping: 5.0 },
        )).id();

        // Car body (main hull)
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.8, 0.6, 3.6))),
            MeshMaterial3d(materials.add(color)),
            Transform::from_xyz(0.0, 0.0, 0.0),
            ChildOf(car_entity),
        ));
    }
}
