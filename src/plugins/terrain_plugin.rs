use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

use crate::services::terrain_service::TerrainService;
use crate::systems::terrain::{
    TerrainConfig, LoadedTerrainConfig,
    load_terrain_config_system, process_loaded_terrain_config_system,
    debug_terrain_config_system,
};

/// Plugin for terrain system management
/// Phase 2: Asset-driven configuration with hot-reload support
pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        // Add RON asset loader for terrain configuration
        app.add_plugins(RonAssetPlugin::<TerrainConfig>::new(&["ron"]));
        
        app
            // Register asset types
            .init_asset::<TerrainConfig>()
            
            // Initialize resources
            .init_resource::<TerrainService>()
            .init_resource::<LoadedTerrainConfig>()
            
            // Asset loading systems (run in Startup for initial load, then Update for hot-reload)
            .add_systems(Startup, (
                setup_terrain_service,
                load_terrain_config_system,
            ))
            .add_systems(Update, (
                process_loaded_terrain_config_system,
                update_terrain_service_with_config,
                debug_terrain_config_system,
            ));
            
        info!("✅ TERRAIN PLUGIN: Initialized with asset-driven configuration");
    }
}

/// System to setup terrain service configuration
fn setup_terrain_service(mut terrain_service: ResMut<TerrainService>) {
    // Phase 2: Configure for compatibility with existing ground detection
    // Asset configuration will override these defaults when loaded
    terrain_service.set_enabled(true);
    terrain_service.set_fallback_height(-0.15); // Match existing terrain height
    
    info!("✅ TERRAIN SERVICE: Configured with fallback defaults (waiting for asset config)");
}

/// System to update terrain service when new configuration is loaded
fn update_terrain_service_with_config(
    loaded_config: Res<LoadedTerrainConfig>,
    mut terrain_service: ResMut<TerrainService>,
) {
    // Only update if we have a new config and service doesn't have it yet
    if let Some(ref config) = loaded_config.config {
        if !terrain_service.has_config() {
            terrain_service.update_config(config.clone());
            info!("✅ TERRAIN SERVICE: Updated with asset configuration");
        }
    }
}
