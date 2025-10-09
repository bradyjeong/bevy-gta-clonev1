// HighSpeed component removed - no longer needed for finite world
use crate::factories::VehicleFactory;
use crate::services::ground_detection::GroundDetectionService;
use crate::systems::spawn_validation::{SpawnRegistry, SpawnValidator, SpawnableType};
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
/// - Ground detection using GroundDetectionService for proper positioning
/// - Spawn validation using SpawnValidator/SpawnRegistry
/// - Consistent bundle patterns similar to DynamicVehicleBundle
/// - Far visibility culling distances for aircraft
/// - Proper collision groups for aircraft physics
pub fn setup_initial_aircraft_unified(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_registry: ResMut<SpawnRegistry>,
    ground_service: Res<GroundDetectionService>,
    _config: Res<GameConfig>,
) {
    // Aircraft spawn positions (well-spaced from other content)
    let aircraft_spawns = [
        (Vec3::new(15.0, 0.0, 15.0), AircraftType::Helicopter),
        (Vec3::new(80.0, 0.0, 120.0), AircraftType::F16),
        (Vec3::new(-150.0, 0.0, 200.0), AircraftType::F16),
        (Vec3::new(250.0, 0.0, -180.0), AircraftType::F16),
        (Vec3::new(-300.0, 0.0, -250.0), AircraftType::F16),
        (Vec3::new(400.0, 0.0, 350.0), AircraftType::F16),
        (Vec3::new(-450.0, 0.0, 100.0), AircraftType::Helicopter),
        (Vec3::new(500.0, 0.0, -450.0), AircraftType::F16),
        (Vec3::new(-200.0, 0.0, -500.0), AircraftType::F16),
        (Vec3::new(600.0, 0.0, 200.0), AircraftType::F16),
        (Vec3::new(-550.0, 0.0, 550.0), AircraftType::Helicopter),
        (Vec3::new(700.0, 0.0, -100.0), AircraftType::F16),
    ];

    let mut spawned_aircraft = Vec::new();

    for (preferred_pos, aircraft_type) in aircraft_spawns {
        if let Some(aircraft_entity) = spawn_aircraft_unified(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut spawn_registry,
            &ground_service,
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

/// Spawn a single aircraft with ground detection and validation using VehicleFactory
fn spawn_aircraft_unified(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    spawn_registry: &mut ResMut<SpawnRegistry>,
    ground_service: &Res<GroundDetectionService>,
    preferred_position: Vec3,
    aircraft_type: AircraftType,
) -> Option<Entity> {
    // Get ground height for proper positioning
    let ground_height = ground_service
        .get_ground_height_simple(Vec2::new(preferred_position.x, preferred_position.z));

    // Calculate spawn position (aircraft spawn at appropriate height above ground)
    let spawn_height = match aircraft_type {
        AircraftType::Helicopter => 1.2, // Helicopter collider half-height (1.2) + small margin
        AircraftType::F16 => 0.9,        // F16 capsule_z(0.9, 10.0) - radius 0.9 = ground clearance
    };

    let spawn_position = Vec3::new(
        preferred_position.x,
        ground_height + spawn_height,
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
