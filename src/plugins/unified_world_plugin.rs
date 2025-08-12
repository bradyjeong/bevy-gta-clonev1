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
use crate::world::{ChunkTables, ChunkTracker, PlacementGrid, RoadNetwork, WorldCoordinator};
use crate::factories::entity_factory_unified::UnifiedEntityFactory;
use crate::events::PlayerChunkChanged;
use crate::systems::world::{PlayerChunkTracker, track_player_chunk_changes, handle_player_chunk_changed};

/// Simplified unified world plugin that coordinates focused world sub-plugins.
/// This follows the simplicity principle by delegating to specialized plugins
/// rather than managing all systems directly.
pub struct UnifiedWorldPlugin;

impl Plugin for UnifiedWorldPlugin {
    fn build(&self, app: &mut App) {
        // Register decomposed resources (P1.1 migration complete)
        app.init_resource::<ChunkTracker>()
            .init_resource::<ChunkTables>()
            .init_resource::<PlacementGrid>()
            .init_resource::<RoadNetwork>()
            .init_resource::<WorldCoordinator>()
            .init_resource::<crate::world::chunk_tracker::ChunkProgress>()
            .init_resource::<UnifiedEntityFactory>()
            .insert_resource(PlayerChunkTracker::new(400.0));
        
        // Register events and observers for player chunk tracking
        app.add_event::<PlayerChunkChanged>()
            .add_observer(handle_player_chunk_changed)
            .add_observer(crate::systems::world::handle_culling_on_player_moved);
        
        app
            // Add focused world plugins
            .add_plugins(TimingPlugin)
            .add_plugins(WorldStreamingPlugin)
            .add_plugins(WorldContentPlugin)
            .add_plugins(WorldLodPlugin)
            .add_plugins(WorldNpcPlugin)
            .add_plugins(WorldDebugPlugin)
            .add_plugins(VegetationInstancingPlugin)
            
            // Player chunk tracking system (runs early to trigger observers)
            .add_systems(Update, track_player_chunk_changes)
            
            // Initialize material factory
            .add_systems(Startup, initialize_material_factory);
    }
}
