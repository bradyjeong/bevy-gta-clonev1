use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::bundles::DynamicVehicleBundle;
use crate::factories::entity_factory_unified::UnifiedEntityFactory;
use crate::services::ground_detection::GroundDetectionService;
use crate::systems::spawn_validation::{SpawnRegistry, SpawnValidator, SpawnableType};
use crate::systems::world::road_network::RoadNetwork;
use crate::systems::world::unified_distance_culling::UnifiedCullable;
use crate::services::distance_cache::MovementTracker;
use crate::setup::vehicles::BugattiColorScheme;
use crate::GameConfig;

/// Unified vehicle setup system that replaces all previous vehicle setup functions
/// - Replaces setup_starter_vehicles (src/setup/starter_vehicles.rs)
/// - Replaces setup_simple_vehicles (src/setup/vehicles.rs)
/// - Replaces setup_luxury_cars (src/setup/environment.rs)
/// 
/// KEY IMPROVEMENTS:
/// - Consistent ground detection using UnifiedEntityFactory patterns
/// - Proper spawn validation and collision detection
/// - Uses DynamicVehicleBundle for all vehicle types
/// - Non-overlapping safe positioning
/// - Unified vehicle spawning logic
pub fn setup_initial_vehicles_unified(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_registry: ResMut<SpawnRegistry>,
    ground_service: Res<GroundDetectionService>,
    _road_network: Option<Res<RoadNetwork>>,
    config: Res<GameConfig>,
) {
    // Initialize UnifiedEntityFactory for consistent spawning
    let mut entity_factory = UnifiedEntityFactory::with_config(config.clone());
    let current_time = 0.0; // Initial spawn time
    
    // Track all spawned vehicles for collision detection
    let mut existing_content: Vec<(Vec3, ContentType, f32)> = Vec::new();
    
    // 1. STARTER VEHICLES (3 vehicles with ground detection - from starter_vehicles.rs)
    let starter_vehicles = setup_starter_vehicles_unified(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut spawn_registry,
        &ground_service,
        &mut entity_factory,
        &mut existing_content,
        current_time,
    );
    
    // 2. SUPERCAR (1 Bugatti from simple_vehicles.rs)
    let supercar = setup_supercar_unified(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut spawn_registry,
        &ground_service,
        &mut entity_factory,
        &mut existing_content,
        current_time,
    );
    
    // 3. LUXURY CARS (5-8 cars with proper validation - from luxury_cars)
    let luxury_cars = setup_luxury_cars_unified(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut spawn_registry,
        &ground_service,
        &mut entity_factory,
        &mut existing_content,
        current_time,
    );
    
    info!("Unified vehicle setup complete - Spawned {} starter vehicles, {} supercar, {} luxury cars", 
        starter_vehicles.len(), if supercar.is_some() { 1 } else { 0 }, luxury_cars.len());
}

/// Setup starter vehicles with ground detection (replaces setup_starter_vehicles)
fn setup_starter_vehicles_unified(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    spawn_registry: &mut ResMut<SpawnRegistry>,
    ground_service: &Res<GroundDetectionService>,
    entity_factory: &mut UnifiedEntityFactory,
    existing_content: &mut Vec<(Vec3, ContentType, f32)>,
    _current_time: f32,
) -> Vec<Entity> {
    let mut spawned_vehicles = Vec::new();
    
    // Safe starter positions (well-spaced from spawn point)
    let starter_positions = [
        Vec3::new(25.0, 0.0, 10.0),   // Near spawn, well spaced
        Vec3::new(-30.0, 0.0, 15.0),  // Different area
        Vec3::new(10.0, 0.0, -25.0),  // Another area
    ];
    
    let car_colors = [
        Color::srgb(1.0, 0.0, 0.0), // Red
        Color::srgb(0.0, 0.0, 1.0), // Blue  
        Color::srgb(0.0, 1.0, 0.0), // Green
    ];
    
    for (i, &preferred_position) in starter_positions.iter().enumerate() {
        let color = car_colors[i % car_colors.len()];
        
        // Use ground detection service for proper positioning
        let vehicle_spawn_pos = Vec2::new(preferred_position.x, preferred_position.z);
        let ground_height = ground_service.get_ground_height_simple(vehicle_spawn_pos);
        let vehicle_y = ground_height + 0.5; // Vehicle collider half-height above ground
        let final_position = Vec3::new(preferred_position.x, vehicle_y, preferred_position.z);
        
        // Validate position using UnifiedEntityFactory
        let validated_position = match entity_factory.validate_position(final_position) {
            Ok(pos) => pos,
            Err(e) => {
                warn!("Failed to validate starter vehicle {} position: {:?}", i, e);
                continue;
            }
        };
        
        // Note: We'll use SpawnValidator to find safe position instead of skipping
        
        // Create vehicle using DynamicVehicleBundle
        let vehicle_entity = commands.spawn((
            DynamicVehicleBundle {
                dynamic_content: DynamicContent { content_type: ContentType::Vehicle },
                car: Car,
                transform: Transform::from_translation(validated_position),
                visibility: Visibility::default(),
                inherited_visibility: InheritedVisibility::VISIBLE,
                view_visibility: ViewVisibility::default(),
                rigid_body: RigidBody::Dynamic,
                collider: Collider::cuboid(1.0, 0.5, 2.0),
                collision_groups: CollisionGroups::new(
                    entity_factory.config.physics.vehicle_group,
                    entity_factory.config.physics.static_group | 
                    entity_factory.config.physics.vehicle_group | 
                    entity_factory.config.physics.character_group
                ),
                velocity: Velocity::default(),
                damping: Damping { linear_damping: 1.0, angular_damping: 5.0 },
                locked_axes: LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
                cullable: UnifiedCullable::vehicle(),
            },
            MovementTracker::new(validated_position, 10.0),
            Name::new(format!("StarterVehicle_{}", i)),
        )).id();
        
        // Add car body as child entity
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.8, 1.0, 3.6))),
            MeshMaterial3d(materials.add(color)),
            Transform::from_xyz(0.0, 0.0, 0.0),
            ChildOf(vehicle_entity),
            InheritedVisibility::VISIBLE,
        ));
        
        // Use spawn validator for safe positioning
        if let Some(safe_position) = SpawnValidator::spawn_entity_safely(
            spawn_registry,
            validated_position,
            SpawnableType::Vehicle,
            vehicle_entity,
        ) {
            // Update transform if position was adjusted
            if safe_position != validated_position {
                if let Ok(mut entity_commands) = commands.get_entity(vehicle_entity) {
                    entity_commands.insert(Transform::from_translation(safe_position));
                }
            }
            
            // Track spawned vehicle
            existing_content.push((safe_position, ContentType::Vehicle, 25.0));
            spawned_vehicles.push(vehicle_entity);
            
            info!("Starter vehicle {} spawned at position: {:?}", i, safe_position);
        } else {
            warn!("Failed to find safe position for starter vehicle {}", i);
            // Register at validated position as fallback
            spawn_registry.register_entity(vehicle_entity, validated_position, SpawnableType::Vehicle);
            existing_content.push((validated_position, ContentType::Vehicle, 25.0));
            spawned_vehicles.push(vehicle_entity);
        }
    }
    
    spawned_vehicles
}

/// Setup supercar with ground detection (replaces Bugatti from setup_simple_vehicles)
fn setup_supercar_unified(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    spawn_registry: &mut ResMut<SpawnRegistry>,
    ground_service: &Res<GroundDetectionService>,
    entity_factory: &mut UnifiedEntityFactory,
    existing_content: &mut Vec<(Vec3, ContentType, f32)>,
    _current_time: f32,
) -> Option<Entity> {
    // Safe supercar position (away from other vehicles and player spawn)
    let preferred_position = Vec3::new(35.0, 0.0, 5.0);
    
    // Use ground detection service
    let vehicle_spawn_pos = Vec2::new(preferred_position.x, preferred_position.z);
    let ground_height = ground_service.get_ground_height_simple(vehicle_spawn_pos);
    let vehicle_y = ground_height + 0.5;
    let final_position = Vec3::new(preferred_position.x, vehicle_y, preferred_position.z);
    
    // Validate position
    let validated_position = match entity_factory.validate_position(final_position) {
        Ok(pos) => pos,
        Err(e) => {
            warn!("Failed to validate supercar position: {:?}", e);
            return None;
        }
    };
    
    // Note: We'll use SpawnValidator to find safe position instead of skipping
    
    // Create Bugatti supercar with premium materials
    let color_scheme = BugattiColorScheme::MidnightBlue;
    let supercar_entity = commands.spawn((
        DynamicVehicleBundle {
            dynamic_content: DynamicContent { content_type: ContentType::Vehicle },
            car: Car,
            transform: Transform::from_translation(validated_position),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::VISIBLE,
            view_visibility: ViewVisibility::default(),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::cuboid(1.1, 0.5, 2.4), // Slightly larger than basic car
            collision_groups: CollisionGroups::new(
                entity_factory.config.physics.vehicle_group,
                entity_factory.config.physics.static_group | 
                entity_factory.config.physics.vehicle_group | 
                entity_factory.config.physics.character_group
            ),
            velocity: Velocity::default(),
            damping: Damping { linear_damping: 1.0, angular_damping: 5.0 },
            locked_axes: LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
            cullable: UnifiedCullable::vehicle(),
        },
        SuperCarBundle::default(),
        MovementTracker::new(validated_position, 10.0),
        Name::new("BugattiSupercar"),
    )).id();
    
    // Supercar body with premium materials
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 1.0, 4.5))),
        MeshMaterial3d(materials.add(color_scheme.get_material())),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ChildOf(supercar_entity),
        InheritedVisibility::VISIBLE,
    ));
    
    // Add carbon fiber accents
    for side in [-1.0, 1.0] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.05, 0.6, 3.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.15, 0.15, 0.15), // Carbon fiber black
                metallic: 0.9,
                perceptual_roughness: 0.1,
                ..default()
            })),
            Transform::from_xyz(side * 1.0, 0.0, 0.0),
            ChildOf(supercar_entity),
            InheritedVisibility::VISIBLE,
        ));
    }
    
    // Register with spawn validator
    if let Some(safe_position) = SpawnValidator::spawn_entity_safely(
        spawn_registry,
        validated_position,
        SpawnableType::Vehicle,
        supercar_entity,
    ) {
        if safe_position != validated_position {
            if let Ok(mut entity_commands) = commands.get_entity(supercar_entity) {
                entity_commands.insert(Transform::from_translation(safe_position));
                info!("Supercar moved to safe position: {:?}", safe_position);
            }
        }
        
        existing_content.push((safe_position, ContentType::Vehicle, 30.0));
        info!("Supercar spawned at position: {:?}", safe_position);
        Some(supercar_entity)
    } else {
        warn!("Failed to find safe position for supercar, using fallback");
        spawn_registry.register_entity(supercar_entity, validated_position, SpawnableType::Vehicle);
        existing_content.push((validated_position, ContentType::Vehicle, 30.0));
        info!("Supercar spawned at fallback position: {:?}", validated_position);
        Some(supercar_entity)
    }
}

/// Setup luxury cars with proper validation (replaces setup_luxury_cars)
fn setup_luxury_cars_unified(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    spawn_registry: &mut ResMut<SpawnRegistry>,
    ground_service: &Res<GroundDetectionService>,
    entity_factory: &mut UnifiedEntityFactory,
    existing_content: &mut Vec<(Vec3, ContentType, f32)>,
    _current_time: f32,
) -> Vec<Entity> {
    let mut spawned_vehicles = Vec::new();
    
    // Luxury car positions (selected safe positions from original list)
    let luxury_positions = [
        Vec3::new(15.0, 0.0, 8.0),    // Near spawn area
        Vec3::new(-8.0, 0.0, 12.0),   // Different area
        Vec3::new(18.0, 0.0, -15.0),  // Another area
        Vec3::new(50.0, 0.0, 0.0),    // Highway position (moved from 35,0 to avoid Bugatti)
        Vec3::new(-40.0, 0.0, 0.0),   // Highway position
        Vec3::new(65.0, 0.0, 55.0),   // District position
        Vec3::new(-70.0, 0.0, 60.0),  // District position
        Vec3::new(120.0, 0.0, 110.0), // Luxury area
    ];
    
    // Luxury Dubai car colors
    let luxury_colors = [
        Color::srgb(1.0, 1.0, 1.0), // Pearl White
        Color::srgb(0.1, 0.1, 0.1), // Jet Black
        Color::srgb(0.8, 0.7, 0.0), // Gold
        Color::srgb(0.7, 0.7, 0.8), // Silver Metallic
        Color::srgb(0.8, 0.1, 0.1), // Ferrari Red
        Color::srgb(0.1, 0.3, 0.8), // Royal Blue
        Color::srgb(0.2, 0.6, 0.2), // British Racing Green
        Color::srgb(0.6, 0.3, 0.8), // Purple
    ];
    
    for (i, &preferred_position) in luxury_positions.iter().enumerate() {
        let color = luxury_colors[i % luxury_colors.len()];
        
        // Use ground detection service for proper positioning
        let vehicle_spawn_pos = Vec2::new(preferred_position.x, preferred_position.z);
        let ground_height = ground_service.get_ground_height_simple(vehicle_spawn_pos);
        let vehicle_y = ground_height + 0.5;
        let final_position = Vec3::new(preferred_position.x, vehicle_y, preferred_position.z);
        
        // Validate position using UnifiedEntityFactory
        let validated_position = match entity_factory.validate_position(final_position) {
            Ok(pos) => pos,
            Err(e) => {
                warn!("Failed to validate luxury car {} position: {:?}", i, e);
                continue;
            }
        };
        
        // Note: We'll use SpawnValidator to find safe position instead of skipping
        
        // Create luxury vehicle using DynamicVehicleBundle
        let vehicle_entity = commands.spawn((
            DynamicVehicleBundle {
                dynamic_content: DynamicContent { content_type: ContentType::Vehicle },
                car: Car,
                transform: Transform::from_translation(validated_position),
                visibility: Visibility::default(),
                inherited_visibility: InheritedVisibility::VISIBLE,
                view_visibility: ViewVisibility::default(),
                rigid_body: RigidBody::Dynamic,
                collider: Collider::cuboid(1.0, 0.5, 2.0),
                collision_groups: CollisionGroups::new(
                    entity_factory.config.physics.vehicle_group,
                    entity_factory.config.physics.static_group | 
                    entity_factory.config.physics.vehicle_group | 
                    entity_factory.config.physics.character_group
                ),
                velocity: Velocity::default(),
                damping: Damping { linear_damping: 1.0, angular_damping: 5.0 },
                locked_axes: LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
                cullable: UnifiedCullable::vehicle(),
            },
            MovementTracker::new(validated_position, 10.0),
            Name::new(format!("LuxuryCar_{}", i)),
        )).id();
        
        // Add luxury car body with premium materials
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.8, 1.0, 3.6))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                metallic: 0.8,
                perceptual_roughness: 0.1,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, 0.0),
            ChildOf(vehicle_entity),
            InheritedVisibility::VISIBLE,
        ));
        
        // Use spawn validator for safe positioning
        if let Some(safe_position) = SpawnValidator::spawn_entity_safely(
            spawn_registry,
            validated_position,
            SpawnableType::Vehicle,
            vehicle_entity,
        ) {
            // Update transform if position was adjusted
            if safe_position != validated_position {
                if let Ok(mut entity_commands) = commands.get_entity(vehicle_entity) {
                    entity_commands.insert(Transform::from_translation(safe_position));
                }
            }
            
            // Track spawned vehicle
            existing_content.push((safe_position, ContentType::Vehicle, 25.0));
            spawned_vehicles.push(vehicle_entity);
            
            info!("Luxury car {} spawned at position: {:?}", i, safe_position);
        } else {
            warn!("Failed to find safe position for luxury car {}", i);
            // Register at validated position as fallback
            spawn_registry.register_entity(vehicle_entity, validated_position, SpawnableType::Vehicle);
            existing_content.push((validated_position, ContentType::Vehicle, 25.0));
            spawned_vehicles.push(vehicle_entity);
        }
    }
    
    spawned_vehicles
}
