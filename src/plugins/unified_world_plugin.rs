use crate::factories::material_factory::initialize_material_factory;
use crate::plugins::{
    TimingPlugin, WorldDebugPlugin, WorldLodPlugin, WorldNpcPlugin, WorldStreamingPlugin,
};
use crate::systems::world::async_chunk_generation::AsyncChunkGenerationPlugin;
use bevy::prelude::*;

/// Simplified unified world plugin that coordinates focused world sub-plugins.
/// This follows the simplicity principle by delegating to specialized plugins
/// rather than managing all systems directly.
///
/// NOW WITH ASYNC GENERATION: Uses AsyncChunkGenerationPlugin for smooth 60+ FPS
pub struct UnifiedWorldPlugin;

impl Plugin for UnifiedWorldPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add focused world plugins
            .add_plugins(TimingPlugin)
            .add_plugins(WorldStreamingPlugin)
            .add_plugins(AsyncChunkGenerationPlugin) // NEW: Async budgeted generation
            .add_plugins(WorldLodPlugin)
            .add_plugins(WorldNpcPlugin)
            .add_plugins(WorldDebugPlugin)
            // Initialize material factory
            .add_systems(Startup, initialize_material_factory);
    }
}
