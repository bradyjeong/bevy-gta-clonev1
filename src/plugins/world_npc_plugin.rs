use crate::systems::world::{
    npc::simple_npc_movement, npc_animation::npc_animation_system, npc_spawn::spawn_new_npc_system,
};
use bevy::prelude::*;

/// Plugin responsible for NPC spawning, behavior, and management
pub struct WorldNpcPlugin;

impl Plugin for WorldNpcPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_new_npc_system)
            .add_systems(Update, simple_npc_movement)
            .add_systems(Update, npc_animation_system.after(simple_npc_movement));
    }
}
