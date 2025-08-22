use crate::systems::world::{migrate_legacy_npcs, simple_npc_movement, spawn_new_npc_system};
use bevy::prelude::*;

/// Plugin responsible for NPC spawning, behavior, and management
pub struct WorldNpcPlugin;

impl Plugin for WorldNpcPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, migrate_legacy_npcs)
            .add_systems(Update, spawn_new_npc_system)
            .add_systems(Update, simple_npc_movement);
    }
}
