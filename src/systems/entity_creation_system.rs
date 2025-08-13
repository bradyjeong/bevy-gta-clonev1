use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::services::{Services, implementations::{DefaultConfigService, DefaultPhysicsService, DefaultAssetService, DefaultLoggingService}};
use crate::services::traits::{ConfigService, PhysicsService, AssetService, LoggingService};
use crate::factories::entity_factory_unified::UnifiedEntityFactory;

/// Example system showing how to use services for entity creation
pub fn service_based_entity_creation_system(
    mut commands: Commands,
    services: Services,
    _factory: Res<UnifiedEntityFactory>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Only create entities when requested
    if !keyboard_input.just_pressed(KeyCode::F10) {
        return;
    }
    
    // Get services using the Services system parameter
    let config_service = services.require::<DefaultConfigService>();
    let physics_service = services.require::<DefaultPhysicsService>();
    let asset_service = services.require::<DefaultAssetService>();
    let logging_service = services.require::<DefaultLoggingService>();
    
    // Use services to get configuration and validate parameters
    let vehicle_config = {
        let config = config_service.read().unwrap();
        config.get_vehicle_config().basic_car.clone()
    };
    
    // Validate spawn position using physics service
    let spawn_position = Vec3::new(10.0, 2.0, 10.0);
    let validated_position = {
        let physics = physics_service.read().unwrap();
        physics.validate_position(spawn_position)
    };
    
    // Validate mass using physics service
    let validated_mass = {
        let physics = physics_service.read().unwrap();
        physics.validate_mass(vehicle_config.mass)
    };
    
    // Get collision groups from physics service
    let (_, vehicle_group, _) = {
        let physics = physics_service.read().unwrap();
        physics.get_collision_groups()
    };
    
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
    
    // Register assets with asset service for future reference
    {
        let mut asset_service_guard = asset_service.write().unwrap();
        asset_service_guard.register_mesh("service_test_car_body".to_string(), mesh_handle.clone());
        asset_service_guard.register_material("service_test_car_material".to_string(), material_handle.clone());
    }
    
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
        .insert(Name::new("ServiceCreatedCar"))
        .id();
    
    // Log the creation using logging service
    {
        let logger = logging_service.read().unwrap();
        logger.log_info(&format!(
            "Created service-based vehicle entity {:?} at position {:?} with mass {:.2}",
            entity, validated_position, validated_mass
        ));
    }
}

/// System demonstrating service-based configuration updates
pub fn service_config_update_system(
    services: Services,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::F11) {
        let config_service = services.require::<DefaultConfigService>();
        let logging_service = services.require::<DefaultLoggingService>();
        
        // Validate and clamp configuration
        {
            let mut config = config_service.write().unwrap();
            config.validate_and_clamp();
        }
        
        // Log configuration validation
        {
            let logger = logging_service.read().unwrap();
            logger.log_info("Configuration validated and clamped via service");
        }
    }
}

/// System demonstrating service-based asset management
pub fn service_asset_cleanup_system(
    services: Services,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::F12) {
        let asset_service = services.require::<DefaultAssetService>();
        let logging_service = services.require::<DefaultLoggingService>();
        
        // Cleanup unused assets
        {
            let mut assets = asset_service.write().unwrap();
            assets.cleanup_unused_assets();
        }
        
        // Log cleanup
        {
            let logger = logging_service.read().unwrap();
            logger.log_info("Asset cleanup requested via service");
        }
    }
}

/// Example of service usage in factory system
pub fn service_based_factory_system(
    _factory: Res<UnifiedEntityFactory>,
    services: Services,
    _time: Res<Time>,
) {
    // This system demonstrates accessing multiple services in a factory context
    let config_service = services.require::<DefaultConfigService>();
    let _physics_service = services.require::<DefaultPhysicsService>();
    
    // Validate factory operations using services
    let world_config = {
        let config = config_service.read().unwrap();
        config.get_world_config().clone()
    };
    
    // Use physics service for any validation needed
    let _max_entities = world_config.building_density * 100.0; // Example calculation
    
    // Services provide consistent access to configuration and validation
    // Factory can now focus on entity creation logic without managing config directly
}
