use bevy::prelude::*;
use crate::systems::movement::{human_player_movement, human_player_animation};
use crate::systems::camera::camera_follow_system;
use crate::systems::interaction::interaction_system;
use crate::systems::audio::{footstep_system, cleanup_footstep_sounds};
use crate::systems::human_behavior::{human_emotional_state_system, human_fidget_system};
use crate::systems::world::debug::debug_game_state;
use crate::systems::player_collision_resolution::{player_collision_resolution_system, player_movement_validation_system};
use game_core::game_state::GameState;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            human_player_movement.run_if(in_state(GameState::Walking)),
            human_player_animation.run_if(in_state(GameState::Walking)),
            human_emotional_state_system.run_if(in_state(GameState::Walking)),
            human_fidget_system.run_if(in_state(GameState::Walking)),
            footstep_system.run_if(in_state(GameState::Walking)),
            cleanup_footstep_sounds,
            camera_follow_system,
            interaction_system,
            debug_game_state,
            (player_collision_resolution_system, player_movement_validation_system).run_if(in_state(GameState::Walking)),
        ));
    }
}
