use bevy::prelude::*;
use crate::plugins::{
    WorldStreamingPlugin,
    WorldContentPlugin,
    WorldLodPlugin,
    WorldNpcPlugin,
    WorldDebugPlugin,
    TimingPlugin,
    VegetationInstancingPlugin,
};
use crate::factories::initialize_material_factory;

#[cfg(feature = "p1_1_decomp")]
use crate::world::{ChunkTracker, PlacementGrid, RoadNetwork, WorldCoordinator};

#[cfg(feature = "world_v2")]
use crate::world::migration::{extract_world_manager, validate_migration, WorldExtractionComplete};

/// Simplified unified world plugin that coordinates focused world sub-plugins.
/// This follows the simplicity principle by delegating to specialized plugins
/// rather than managing all systems directly.
pub struct UnifiedWorldPlugin;

impl Plugin for UnifiedWorldPlugin {
    fn build(&self, app: &mut App) {
        // Register new decomposed resources (if feature enabled)
        #[cfg(feature = "p1_1_decomp")]
        {
            app.init_resource::<ChunkTracker>()
                .init_resource::<PlacementGrid>()
                .init_resource::<RoadNetwork>()
                .init_resource::<WorldCoordinator>();
        }
        
        // Add migration system for world_v2 feature
        #[cfg(feature = "world_v2")]
        {
            app.add_event::<WorldExtractionComplete>()
                .add_systems(
                    Startup,
                    (
                        extract_world_manager,
                        validate_migration.after(extract_world_manager)
                    )
                );
        }
        
        app
            // Add focused world plugins
            .add_plugins(TimingPlugin)
            .add_plugins(WorldStreamingPlugin)
            .add_plugins(WorldContentPlugin)
            .add_plugins(WorldLodPlugin)
            .add_plugins(WorldNpcPlugin)
            .add_plugins(WorldDebugPlugin)
            .add_plugins(VegetationInstancingPlugin)
            
            // Initialize material factory
            .add_systems(Startup, initialize_material_factory);
    }
}
