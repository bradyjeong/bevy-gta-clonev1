use crate::GameConfig;
use crate::components::vehicles::VehicleType;
use crate::factories::VehicleFactory;
use bevy::prelude::*;

/// Simple yacht spawning system for testing the new water physics
pub fn spawn_test_yacht(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<GameConfig>,
) {
    let yacht_position = Vec3::new(300.0, 1.0, 300.0); // Center of lake
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
