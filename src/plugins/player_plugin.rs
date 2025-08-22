use crate::systems::audio::{cleanup_footstep_sounds, footstep_system};
use crate::systems::camera::camera_follow_system;
use crate::systems::input::asset_based_input_mapping_system;
use crate::systems::interaction::interaction_system;
use crate::systems::movement::{
    PlayerInputData, animation_flag_system, human_player_animation, read_input_system,
    velocity_apply_system,
};
use bevy::prelude::*;

use crate::game_state::GameState;
use crate::systems::debug::debug_game_state;
use crate::systems::player_collision_resolution::{
    player_collision_resolution_system, player_movement_validation_system,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInputData>().add_systems(
            Update,
            (
                asset_based_input_mapping_system.before(read_input_system),
                read_input_system.run_if(in_state(GameState::Walking)),
                velocity_apply_system
                    .after(read_input_system)
                    .run_if(in_state(GameState::Walking)),
                animation_flag_system
                    .after(velocity_apply_system)
                    .run_if(in_state(GameState::Walking)),
                human_player_animation
                    .after(animation_flag_system)
                    .run_if(in_state(GameState::Walking)),
                footstep_system.run_if(in_state(GameState::Walking)),
                cleanup_footstep_sounds,
                camera_follow_system,
                interaction_system,
                debug_game_state,
                (
                    player_collision_resolution_system,
                    player_movement_validation_system,
                )
                    .run_if(in_state(GameState::Walking)),
            ),
        );
    }
}
