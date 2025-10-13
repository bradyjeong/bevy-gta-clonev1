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
    // Yacht spawns on left terrain island lake (X=-1500+300=-1200, Z=300)
    let left_terrain_x = -1500.0;
    let yacht_position = Vec3::new(left_terrain_x + 300.0, SEA_LEVEL, 300.0);
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
