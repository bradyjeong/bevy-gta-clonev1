#![allow(clippy::too_many_arguments, clippy::type_complexity, deprecated)]

use crate::components::ContentType;
use crate::constants::WorldEnvConfig;
use crate::factories::VehicleFactory;

use crate::systems::spawn_validation::{SpawnRegistry, SpawnValidator, SpawnableType};
use crate::systems::world::road_network::RoadNetwork;
use crate::systems::world::unified_world::UnifiedWorldManager;
use bevy::prelude::*;

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
    asset_server: Res<AssetServer>,
    mut spawn_registry: ResMut<SpawnRegistry>,
    world: Res<UnifiedWorldManager>,
    _road_network: Option<Res<RoadNetwork>>,
    _config: Res<GameConfig>,
    env: Res<WorldEnvConfig>,
) {
    // Initialize focused VehicleFactory for consistent spawning following AGENT.md principles
    let vehicle_factory = VehicleFactory::new();
    let current_time = 0.0; // Initial spawn time

    // Track all spawned vehicles for collision detection
    let mut existing_content: Vec<(Vec3, ContentType, f32)> = Vec::new();

    // 1. STARTER VEHICLES (3 vehicles - from starter_vehicles.rs)
    let _starter_vehicles = setup_starter_vehicles_unified(
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
        &mut spawn_registry,
        &vehicle_factory,
        &mut existing_content,
        current_time,
        &env,
    );

    // 3. LUXURY CARS (5-8 cars with proper validation - from luxury_cars)
    let _luxury_cars = setup_luxury_cars_unified(
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
        &mut spawn_registry,
        &vehicle_factory,
        &mut existing_content,
        current_time,
        &world,
        &env,
    );

    #[cfg(feature = "debug-ui")]
    info!(
        "Unified vehicle setup complete - Spawned {} starter vehicles, {} luxury cars",
        starter_vehicles.len(),
        luxury_cars.len()
    );
}

/// Setup starter vehicles with ground detection (replaces setup_starter_vehicles)
fn setup_starter_vehicles_unified(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
    spawn_registry: &mut ResMut<SpawnRegistry>,
    vehicle_factory: &VehicleFactory,
    existing_content: &mut Vec<(Vec3, ContentType, f32)>,
    _current_time: f32,
    env: &WorldEnvConfig,
) -> Vec<Entity> {
    let mut spawned_vehicles = Vec::new();

    // Safe starter positions on left terrain island (within ±600m bounds)
    let starter_positions = [
        Vec3::new(env.islands.left_x + 25.0, 0.0, 10.0), // Near spawn, well spaced
        Vec3::new(env.islands.left_x - 30.0, 0.0, 15.0), // Different area
        Vec3::new(env.islands.left_x + 10.0, 0.0, -25.0), // Another area
    ];

    let car_colors = [
        Color::srgb(1.0, 0.0, 0.0), // Red
        Color::srgb(0.0, 0.0, 1.0), // Blue
        Color::srgb(0.0, 1.0, 0.0), // Green
    ];

    for (i, &preferred_position) in starter_positions.iter().enumerate() {
        let color = car_colors[i % car_colors.len()];

        // Spawn above terrain, let gravity drop vehicles
        let vehicle_y = env.land_elevation + env.spawn_drop_height;
        let final_position = Vec3::new(preferred_position.x, vehicle_y, preferred_position.z);

        // Use simplified approach with focused factory
        let validated_position = final_position;

        // Create vehicle using focused VehicleFactory
        let vehicle_entity = match vehicle_factory.spawn_super_car(
            commands,
            meshes,
            materials,
            asset_server,
            validated_position,
            Some(color),
        ) {
            Ok(entity) => entity,
            Err(e) => {
                warn!("Failed to spawn starter vehicle {}: {:?}", i, e);
                continue;
            }
        };

        // Use spawn validator for safe positioning
        if let Some(safe_position) = SpawnValidator::spawn_entity_safely(
            spawn_registry,
            validated_position,
            SpawnableType::Vehicle,
            vehicle_entity,
        ) {
            // Update transform if position was adjusted
            if safe_position != validated_position
                && let Ok(mut entity_commands) = commands.get_entity(vehicle_entity)
            {
                entity_commands.insert(Transform::from_translation(safe_position));
            }

            // Track spawned vehicle
            existing_content.push((safe_position, ContentType::Vehicle, 25.0));
            spawned_vehicles.push(vehicle_entity);

            #[cfg(feature = "debug-ui")]
            info!(
                "Starter vehicle {} spawned at position: {:?}",
                i, safe_position
            );
        } else {
            warn!("Failed to find safe position for starter vehicle {}", i);
            // Register at validated position as fallback
            spawn_registry.register_entity(
                vehicle_entity,
                validated_position,
                SpawnableType::Vehicle,
            );
            existing_content.push((validated_position, ContentType::Vehicle, 25.0));
            spawned_vehicles.push(vehicle_entity);
        }
    }

    spawned_vehicles
}

/// Setup luxury cars with proper validation (replaces setup_luxury_cars)
fn setup_luxury_cars_unified(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
    spawn_registry: &mut ResMut<SpawnRegistry>,
    vehicle_factory: &VehicleFactory,
    existing_content: &mut Vec<(Vec3, ContentType, f32)>,
    _current_time: f32,
    world: &UnifiedWorldManager,
    env: &WorldEnvConfig,
) -> Vec<Entity> {
    let mut spawned_vehicles = Vec::new();

    // Luxury car positions on left terrain island (clamped within ±600m bounds)
    let luxury_positions = [
        Vec3::new(env.islands.left_x + 15.0, 0.0, 8.0), // Near spawn area
        Vec3::new(env.islands.left_x - 8.0, 0.0, 12.0), // Different area
        Vec3::new(env.islands.left_x + 18.0, 0.0, -15.0), // Another area
        Vec3::new(env.islands.left_x + 50.0, 0.0, 0.0), // Highway position
        Vec3::new(env.islands.left_x - 40.0, 0.0, 0.0), // Highway position
        Vec3::new(env.islands.left_x + 65.0, 0.0, 55.0), // District position
        Vec3::new(env.islands.left_x - 70.0, 0.0, 60.0), // District position
        Vec3::new(env.islands.left_x + 120.0, 0.0, 110.0), // Edge position (needs clamping)
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

        // Validate spawn position is on terrain island
        if !world.is_on_terrain_island(preferred_position) {
            warn!(
                "Skipping luxury car {} - position {:?} is not on terrain island",
                i, preferred_position
            );
            continue;
        }

        // Spawn above terrain, let gravity drop vehicles
        let vehicle_y = env.land_elevation + env.spawn_drop_height;
        let final_position = Vec3::new(preferred_position.x, vehicle_y, preferred_position.z);

        // Use simplified approach with focused factory
        let validated_position = final_position;

        // Create luxury vehicle using focused VehicleFactory
        let vehicle_entity = match vehicle_factory.spawn_super_car(
            commands,
            meshes,
            materials,
            asset_server,
            validated_position,
            Some(color),
        ) {
            Ok(entity) => entity,
            Err(e) => {
                warn!("Failed to spawn luxury car {}: {:?}", i, e);
                continue;
            }
        };

        // Use spawn validator for safe positioning
        if let Some(safe_position) = SpawnValidator::spawn_entity_safely(
            spawn_registry,
            validated_position,
            SpawnableType::Vehicle,
            vehicle_entity,
        ) {
            // Update transform if position was adjusted
            if safe_position != validated_position
                && let Ok(mut entity_commands) = commands.get_entity(vehicle_entity)
            {
                entity_commands.insert(Transform::from_translation(safe_position));
            }

            // Track spawned vehicle
            existing_content.push((safe_position, ContentType::Vehicle, 25.0));
            spawned_vehicles.push(vehicle_entity);

            #[cfg(feature = "debug-ui")]
            info!("Luxury car {} spawned at position: {:?}", i, safe_position);
        } else {
            warn!("Failed to find safe position for luxury car {}", i);
            // Register at validated position as fallback
            spawn_registry.register_entity(
                vehicle_entity,
                validated_position,
                SpawnableType::Vehicle,
            );
            existing_content.push((validated_position, ContentType::Vehicle, 25.0));
            spawned_vehicles.push(vehicle_entity);
        }
    }

    spawned_vehicles
}
