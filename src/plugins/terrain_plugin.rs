use bevy::prelude::*;

use crate::services::terrain_service::TerrainService;

/// Plugin for terrain system management
/// Phase 1: Service foundation with flat fallback
pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize terrain service resource
            .init_resource::<TerrainService>()
            // Future phases will add terrain generation and management systems here
            .add_systems(Startup, setup_terrain_service);
            
        info!("✅ TERRAIN PLUGIN: Initialized terrain service");
    }
}

/// System to setup terrain service configuration
fn setup_terrain_service(mut terrain_service: ResMut<TerrainService>) {
    // Phase 1: Configure for compatibility with existing ground detection
    terrain_service.set_enabled(true);
    terrain_service.set_fallback_height(-0.15); // Match existing terrain height
    
    info!("✅ TERRAIN SERVICE: Configured with flat fallback at height {}", 
          terrain_service.fallback_height);
}
