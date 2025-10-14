// HighSpeed component removed - no longer needed for finite world
use crate::constants::{LAND_ELEVATION, LEFT_ISLAND_X, SPAWN_DROP_HEIGHT};
use crate::factories::VehicleFactory;

use crate::systems::spawn_validation::{SpawnRegistry, SpawnValidator, SpawnableType};
use crate::systems::world::unified_world::UnifiedWorldManager;
use bevy::prelude::*;

use crate::GameConfig;

/// Aircraft types supported by the unified system
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AircraftType {
    Helicopter, // Complex multi-component aircraft
    F16,        // Simple fighter jet
}

/// Unified aircraft setup system that replaces individual aircraft setup functions
/// - Replaces setup_simple_helicopter (src/setup/vehicles.rs line 219)
/// - Replaces setup_simple_f16 (src/setup/vehicles.rs line 310)
///
/// KEY IMPROVEMENTS:
/// - Island-aware spawn validation for proper positioning
/// - Spawn validation using SpawnValidator/SpawnRegistry
/// - Consistent bundle patterns similar to DynamicVehicleBundle
/// - Far visibility culling distances for aircraft
/// - Proper collision groups for aircraft physics
pub fn setup_initial_aircraft_unified(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_registry: ResMut<SpawnRegistry>,
    world: Res<UnifiedWorldManager>,
    _config: Res<GameConfig>,
) {
    // Aircraft spawn positions on left terrain island (within 600m radius flat terrain)
    // Max distance from island center: ~500m to stay safely on plateau
    let aircraft_spawns = [
        (
            Vec3::new(LEFT_ISLAND_X + 15.0, 0.0, 15.0),
            AircraftType::Helicopter,
        ),
        (
            Vec3::new(LEFT_ISLAND_X + 80.0, 0.0, 120.0),
            AircraftType::F16,
        ),
        (
            Vec3::new(LEFT_ISLAND_X - 150.0, 0.0, 200.0),
            AircraftType::F16,
        ),
        (
            Vec3::new(LEFT_ISLAND_X + 250.0, 0.0, -180.0),
            AircraftType::F16,
        ),
        (
            Vec3::new(LEFT_ISLAND_X - 300.0, 0.0, -250.0),
            AircraftType::F16,
        ),
        (
            Vec3::new(LEFT_ISLAND_X + 350.0, 0.0, 250.0),
            AircraftType::F16,
        ),
        (
            Vec3::new(LEFT_ISLAND_X - 400.0, 0.0, 100.0),
            AircraftType::Helicopter,
        ),
        (
            Vec3::new(LEFT_ISLAND_X + 450.0, 0.0, -350.0),
            AircraftType::F16,
        ),
        (
            Vec3::new(LEFT_ISLAND_X - 200.0, 0.0, -400.0),
            AircraftType::F16,
        ),
        (
            Vec3::new(LEFT_ISLAND_X + 480.0, 0.0, 150.0),
            AircraftType::F16,
        ),
        (
            Vec3::new(LEFT_ISLAND_X - 500.0, 0.0, 200.0),
            AircraftType::Helicopter,
        ),
        (
            Vec3::new(LEFT_ISLAND_X + 300.0, 0.0, -450.0),
            AircraftType::F16,
        ),
    ];

    let mut spawned_aircraft = Vec::new();

    for (preferred_pos, aircraft_type) in aircraft_spawns {
        // Validate spawn position is on terrain island
        if !world.is_on_terrain_island(preferred_pos) {
            warn!(
                "Skipping {:?} - position {:?} is not on terrain island",
                aircraft_type, preferred_pos
            );
            continue;
        }

        if let Some(aircraft_entity) = spawn_aircraft_unified(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut spawn_registry,
            preferred_pos,
            aircraft_type,
        ) {
            spawned_aircraft.push(aircraft_entity);
        }
    }

    info!(
        "Unified aircraft setup complete - Spawned {} aircraft",
        spawned_aircraft.len()
    );
}

/// Spawn a single aircraft with validation using VehicleFactory
fn spawn_aircraft_unified(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    spawn_registry: &mut ResMut<SpawnRegistry>,
    preferred_position: Vec3,
    aircraft_type: AircraftType,
) -> Option<Entity> {
    // Spawn above terrain, let gravity drop aircraft
    let spawn_position = Vec3::new(
        preferred_position.x,
        LAND_ELEVATION + SPAWN_DROP_HEIGHT,
        preferred_position.z,
    );

    // Validate spawn position
    let safe_position = SpawnValidator::spawn_entity_safely(
        spawn_registry,
        spawn_position,
        SpawnableType::Vehicle,
        Entity::PLACEHOLDER, // Will be updated after spawn
    );

    if let Some(validated_position) = safe_position {
        // Use VehicleFactory for consistent vehicle spawning
        let vehicle_factory = VehicleFactory::new();

        let aircraft_entity = match aircraft_type {
            AircraftType::Helicopter => vehicle_factory
                .spawn_helicopter(commands, meshes, materials, validated_position, None)
                .expect("Failed to spawn helicopter"),
            AircraftType::F16 => vehicle_factory
                .spawn_f16(commands, meshes, materials, validated_position, None)
                .expect("Failed to spawn F16"),
        };

        // Update spawn registry with actual entity
        spawn_registry.update_entity_position(aircraft_entity, validated_position);

        Some(aircraft_entity)
    } else {
        warn!(
            "Failed to find safe spawn position for {:?} aircraft",
            aircraft_type
        );
        None
    }
}
