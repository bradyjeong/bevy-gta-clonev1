use crate::GameConfig;
use crate::components::vehicles::VehicleType;
use crate::constants::WorldEnvConfig;
use crate::factories::VehicleFactory;
use bevy::prelude::*;

/// Simple yacht spawning system for testing the new water physics
pub fn spawn_test_yacht(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    config: Res<GameConfig>,
    env: Res<WorldEnvConfig>,
) {
    let yacht_position = Vec3::new(
        config.world_env.islands.left_x
            + config.world_env.terrain.half_size
            + config.world_env.terrain.beach_width,
        env.sea_level,
        0.0,
    );

    let vehicle_factory = VehicleFactory::with_config(config.clone());

    match vehicle_factory.spawn_vehicle_by_type(
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
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
