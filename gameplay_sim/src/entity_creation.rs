use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use game_core::prelude::*;
use crate::services::simple_services::{ConfigService, PhysicsService};
use crate::factories::entity_factory_unified::UnifiedEntityFactory;

/// Example system showing proper Bevy Resource usage (no ServiceContainer)
pub fn bevy_resource_entity_creation_system(
    mut commands: Commands,
    config_service: Res<ConfigService>,
    physics_service: Res<PhysicsService>,
    _factory: Res<UnifiedEntityFactory>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Only create entities when requested
    if !keyboard_input.just_pressed(KeyCode::F10) {
        return;
    }
    
    // Get configuration directly from Bevy Resource
    let vehicle_config = &config_service.get_config().vehicles.basic_car;
    
    // Validate spawn position using physics service
    let spawn_position = Vec3::new(10.0, 2.0, 10.0);
    let validated_position = physics_service.validate_position(spawn_position);
    
    // Validate mass using physics service
    let validated_mass = physics_service.validate_mass(vehicle_config.mass);
    
    // Get collision groups from physics service
    let (_, vehicle_group, _) = physics_service.get_collision_groups();
    
    // Create mesh and material
    let mesh_handle = meshes.add(Cuboid::new(
        vehicle_config.body_size.x,
        vehicle_config.body_size.y,
        vehicle_config.body_size.z,
    ));
    
    let material_handle = materials.add(StandardMaterial {
        base_color: vehicle_config.default_color,
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
            vehicle_config.collider_size.x / 2.0,
            vehicle_config.collider_size.y / 2.0,
            vehicle_config.collider_size.z / 2.0,
        ))
        .insert(CollisionGroups::new(vehicle_group, Group::ALL))
        .insert(AdditionalMassProperties::Mass(validated_mass))
        .insert(Velocity::zero())
        .insert(Damping {
            linear_damping: vehicle_config.linear_damping,
            angular_damping: vehicle_config.angular_damping,
        })
        .insert(Cullable::new(150.0)) // Vehicle culling distance
        .insert(Name::new("BevyResourceCreatedCar"))
        .id();
    
    // Use standard Bevy logging instead of LoggingService
    info!(
        "Created Bevy resource-based vehicle entity {:?} at position {:?} with mass {:.2}",
        entity, validated_position, validated_mass
    );
}

/// System demonstrating Bevy Resource-based configuration updates
pub fn bevy_resource_config_update_system(
    mut config_service: ResMut<ConfigService>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::F11) {
        // Validate and clamp configuration directly
        config_service.validate_and_clamp();
        
        // Use standard Bevy logging
        info!("Configuration validated and clamped via Bevy Resource");
    }
}

/// System demonstrating Bevy's built-in asset management
pub fn bevy_asset_cleanup_system(
    _asset_server: Res<AssetServer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::F12) {
        // Bevy handles asset cleanup automatically
        // For custom asset cleanup, use Bevy's built-in systems
        info!("Asset cleanup handled by Bevy's built-in systems");
    }
}

/// Example of Bevy Resource usage in factory system
pub fn bevy_resource_factory_system(
    _factory: Res<UnifiedEntityFactory>,
    config_service: Res<ConfigService>,
    _physics_service: Res<PhysicsService>,
    _time: Res<Time>,
) {
    // This system demonstrates accessing Bevy Resources directly
    let world_config = &config_service.get_config().world;
    
    // Use physics service for any validation needed
    let _max_entities = world_config.building_density * 100.0; // Example calculation
    
    // Direct Bevy Resource access is cleaner and more performant
    // No need for service containers or trait objects
}
