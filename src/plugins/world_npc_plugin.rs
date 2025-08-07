use bevy::prelude::*;
use crate::systems::world::{
    optimized_npc_movement,
    migrate_legacy_npcs,
    spawn_new_npc_system,
};

/// Plugin responsible for NPC spawning, behavior, and management
pub struct WorldNpcPlugin;

impl Plugin for WorldNpcPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, migrate_legacy_npcs)
            .add_systems(Update, spawn_new_npc_system)
            .add_systems(Update, optimized_npc_movement);
    }
}
