use crate::GameConfig;
use crate::components::vehicles::VehicleType;
use crate::constants::SEA_LEVEL;
use crate::factories::VehicleFactory;
use bevy::prelude::*;

/// Simple yacht spawning system for testing the new water physics
pub fn spawn_test_yacht(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<GameConfig>,
) {
    // Yacht spawns near LEFT island shore for easy player access
    // Position: just east of LEFT island center, accessible from beach
    use crate::constants::{LEFT_ISLAND_X, TERRAIN_HALF_SIZE};
    let yacht_position = Vec3::new(
        LEFT_ISLAND_X + TERRAIN_HALF_SIZE + 150.0, // 150m offshore from east beach
        SEA_LEVEL,
        0.0,
    );

    let vehicle_factory = VehicleFactory::with_config(config.clone());

    match vehicle_factory.spawn_vehicle_by_type(
        &mut commands,
        &mut meshes,
        &mut materials,
        VehicleType::Yacht,
        yacht_position,
        Some(Color::srgb(0.9, 0.9, 1.0)),
    ) {
        Ok(entity) => {
            info!(
                "Spawned test yacht at position: {:?} (Entity: {:?})",
                yacht_position, entity
            );
        }
        Err(e) => {
            warn!("Failed to spawn test yacht: {:?}", e);
        }
    }
}
