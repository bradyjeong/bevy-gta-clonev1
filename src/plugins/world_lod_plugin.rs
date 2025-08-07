use bevy::prelude::*;
use crate::systems::world::{
    master_unified_lod_system,
    master_lod_performance_monitor,
    initialize_master_lod_system,
    adaptive_lod_system,
    npc_lod_system,
    unified_cleanup_system,
};

/// Plugin responsible for Level-of-Detail (LOD) management and performance optimization
pub struct WorldLodPlugin;

impl Plugin for WorldLodPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, initialize_master_lod_system)
            .add_systems(Update, (
                master_unified_lod_system,
                npc_lod_system,
                adaptive_lod_system,
            ).chain())
            .add_systems(Update, (
                master_lod_performance_monitor,
                unified_cleanup_system,
            ).chain());
    }
}
