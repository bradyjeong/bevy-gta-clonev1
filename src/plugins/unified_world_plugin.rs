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
use crate::world::{ChunkTables, ChunkTracker, PlacementGrid, RoadNetwork, WorldCoordinator};

/// Simplified unified world plugin that coordinates focused world sub-plugins.
/// This follows the simplicity principle by delegating to specialized plugins
/// rather than managing all systems directly.
pub struct UnifiedWorldPlugin;

impl Plugin for UnifiedWorldPlugin {
    fn build(&self, app: &mut App) {
        // Register new decomposed resources (always enabled now)
        #[cfg(feature = "p1_1_decomp")]
        {
            app.init_resource::<ChunkTracker>()
                .init_resource::<ChunkTables>()
                .init_resource::<PlacementGrid>()
                .init_resource::<RoadNetwork>()
                .init_resource::<WorldCoordinator>();
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
