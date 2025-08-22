use crate::factories::initialize_material_factory;
use crate::plugins::{
    TimingPlugin, WorldContentPlugin, WorldDebugPlugin, WorldLodPlugin, WorldNpcPlugin,
    WorldStreamingPlugin,
};
use bevy::prelude::*;

/// Simplified unified world plugin that coordinates focused world sub-plugins.
/// This follows the simplicity principle by delegating to specialized plugins
/// rather than managing all systems directly.
pub struct UnifiedWorldPlugin;

impl Plugin for UnifiedWorldPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add focused world plugins
            .add_plugins(TimingPlugin)
            .add_plugins(WorldStreamingPlugin)
            .add_plugins(WorldContentPlugin)
            .add_plugins(WorldLodPlugin)
            .add_plugins(WorldNpcPlugin)
            .add_plugins(WorldDebugPlugin)
            // Initialize material factory
            .add_systems(Startup, initialize_material_factory);
    }
}
