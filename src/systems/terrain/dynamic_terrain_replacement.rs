use bevy::prelude::*;
use crate::components::DynamicTerrain;
use crate::systems::terrain::{LoadedTerrainConfig, spawn_heightmap_terrain};

/// System to replace flat terrain with heightmap terrain once configuration loads
/// 
/// This solves the timing issue where world setup happens before asset loading completes.
/// Instead of falling back to a flat plane forever, we monitor for the terrain configuration
/// to load and then replace the flat terrain with the proper heightmap terrain.
pub fn replace_terrain_when_config_loads(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    loaded_config: Res<LoadedTerrainConfig>,
    terrain_query: Query<Entity, With<DynamicTerrain>>,
    mut terrain_replaced: Local<bool>,
) {
    // Only run once when configuration is loaded and terrain hasn't been replaced yet
    if !*terrain_replaced && loaded_config.config.is_some() {
        info!("Configuration loaded! Replacing flat terrain with heightmap terrain");
        
        // Remove existing flat terrain
        for terrain_entity in terrain_query.iter() {
            info!("Removing flat terrain entity");
            commands.entity(terrain_entity).despawn();
        }
        
        // Spawn new heightmap terrain
        if let Some(ref config) = loaded_config.config {
            spawn_heightmap_terrain(&mut commands, &mut meshes, &mut materials, config);
            info!("✅ Successfully replaced flat terrain with heightmap terrain!");
        }
        
        *terrain_replaced = true;
    }
}
