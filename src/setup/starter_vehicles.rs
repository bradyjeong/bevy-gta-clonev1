use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::bundles::VehicleVisibilityBundle;
use crate::systems::spawn_validation::{SpawnRegistry, SpawnValidator, SpawnableType};
use crate::services::ground_detection::GroundDetectionService;

pub fn setup_starter_vehicles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_registry: ResMut<SpawnRegistry>,
    ground_service: Res<GroundDetectionService>,
) {
    // Add a few starter vehicles in non-overlapping positions
    let starter_positions = [
        Vec3::new(25.0, 0.0, 10.0),   // Near spawn, well spaced - Y will be calculated
        Vec3::new(-30.0, 0.0, 15.0),  // Different area - Y will be calculated
        Vec3::new(10.0, 0.0, -25.0),  // Another area - Y will be calculated
    ];
    
    let car_colors = [
        Color::srgb(1.0, 0.0, 0.0), // Red
        Color::srgb(0.0, 0.0, 1.0), // Blue  
        Color::srgb(0.0, 1.0, 0.0), // Green
    ];
    
    for (i, &preferred_position) in starter_positions.iter().enumerate() {
        let color = car_colors[i % car_colors.len()];
        
        // Calculate proper ground position for vehicle spawn
        let vehicle_spawn_pos = Vec2::new(preferred_position.x, preferred_position.z);
        let ground_height = ground_service.get_ground_height_simple(vehicle_spawn_pos);
        let vehicle_y = ground_height + 0.5; // Vehicle collider half-height above ground
        
        println!("DEBUG: Vehicle {} spawn - ground height: {:.3}, final Y: {:.3}", i, ground_height, vehicle_y);
        
        // Create car parent entity with physics
        let car_entity = commands.spawn((
            Car,
            RigidBody::Dynamic,
            Collider::cuboid(1.0, 0.5, 2.0),  // Half-height = 0.5, total height = 1.0
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
            Velocity::zero(),
            Transform::from_xyz(preferred_position.x, vehicle_y, preferred_position.z),
            VehicleVisibilityBundle::default(),
            CollisionGroups::new(VEHICLE_GROUP, STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP),
            Damping { linear_damping: 1.0, angular_damping: 5.0 },
        )).id();
        
        // Use the calculated ground position for registration
        let final_position = Vec3::new(preferred_position.x, vehicle_y, preferred_position.z);
        
        // Find safe spawn position and register
        if let Some(safe_position) = SpawnValidator::spawn_entity_safely(
            &mut spawn_registry,
            final_position,
            SpawnableType::Vehicle,
            car_entity,
        ) {
            // Update transform if position changed
            if safe_position != final_position {
                if let Ok(mut entity_commands) = commands.get_entity(car_entity) {
                    entity_commands.insert(Transform::from_translation(safe_position));
                    info!("Starter vehicle {} moved to safe position: {:?}", i, safe_position);
                }
            }
        } else {
            warn!("Failed to find safe position for starter vehicle {}, using fallback", i);
            // Register at calculated position anyway as fallback
            spawn_registry.register_entity(car_entity, final_position, SpawnableType::Vehicle);
        }

        // Car body (main hull) - Fixed: height matches collider
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.8, 1.0, 3.6))),  // Fixed: height 1.0 matches collider
            MeshMaterial3d(materials.add(color)),
            Transform::from_xyz(0.0, 0.0, 0.0),
            InheritedVisibility::VISIBLE,
            ChildOf(car_entity),
        ));
    }
}
