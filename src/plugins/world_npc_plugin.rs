use bevy::prelude::*;
use crate::systems::world::{
    simple_npc_movement,
    migrate_legacy_npcs,
    on_npc_spawn_request,
};

/// Plugin responsible for NPC spawning, behavior, and management
/// Now uses observer pattern for reactive spawning (architectural_shift.md §59-63)
pub struct WorldNpcPlugin;

impl Plugin for WorldNpcPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, migrate_legacy_npcs)
            // OBSERVER-BASED: Responds to ChunkLoaded events instead of timer polling
            .add_observer(on_npc_spawn_request)
            .add_systems(Update, simple_npc_movement);
    }
}
