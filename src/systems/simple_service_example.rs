use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::services::simple_services::{ConfigService, PhysicsService, EnhancedTimingService};

/// Example system showing service injection pattern with simple services
pub fn service_example_vehicle_creation(
    mut commands: Commands,
    config_service: Res<ConfigService>,
    physics_service: Res<PhysicsService>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Only create when F10 is pressed
    if !keyboard_input.just_pressed(KeyCode::F10) {
        return;
    }
    
    // Get vehicle configuration from config service
    let _vehicle_config = &config_service.get_config().gameplay.vehicle;
    
    // Validate spawn position using physics service
    let spawn_position = Vec3::new(15.0, 2.0, 15.0);
    let validated_position = physics_service.validate_position(spawn_position);
    
    // Validate mass using physics service  
    let validated_mass = physics_service.validate_mass(1500.0); // Default car mass
    
    // Get collision groups from physics service
    let (_, vehicle_group, _) = physics_service.get_collision_groups();
    
    // Create mesh and material
    let mesh_handle = meshes.add(Cuboid::new(
        2.0, // Default car width
        0.7, // Default car height  
        4.0, // Default car length
    ));
    
    let material_handle = materials.add(StandardMaterial {
        base_color: bevy::color::Color::srgb(0.8, 0.2, 0.2), // Default red color
        ..default()
    });
    
    // Create the entity using validated parameters
    let entity = commands.spawn_empty()
        .insert(Mesh3d(mesh_handle))
        .insert(MeshMaterial3d(material_handle))
        .insert(Transform::from_translation(validated_position))
        .insert(Car)
        .insert(ActiveEntity)
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(
            2.0, // Default car width
            0.7, // Default car height
            4.0, // Default car length
        ))
        .insert(CollisionGroups::new(vehicle_group, Group::ALL))
        .insert(AdditionalMassProperties::Mass(validated_mass))
        .insert(Velocity::zero())
        .insert(Damping {
            linear_damping: 1.0,
            angular_damping: 5.0,
        })
        .insert(Cullable::new(150.0))
        .insert(Name::new("ServiceCreatedCar"))
        .id();
    
    info!(
        "🚗 SERVICE EXAMPLE: Created vehicle entity {:?} at {:?} with mass {:.2}",
        entity, validated_position, validated_mass
    );
}

/// Example system showing configuration validation using services
pub fn service_example_config_validation(
    mut config_service: ResMut<ConfigService>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::F11) {
        config_service.validate_and_clamp();
        info!("🔧 SERVICE EXAMPLE: Configuration validated and clamped");
    }
}

/// Example system showing timing service usage
pub fn service_example_timing_check(
    timing_service: ResMut<EnhancedTimingService>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::F12) {
        let current_time = timing_service.current_time();
        let delta_time = timing_service.delta_time();
        
        info!(
            "⏱️ SERVICE EXAMPLE: Current time: {:.2}s, Delta: {:.4}s",
            current_time, delta_time
        );
    }
}
