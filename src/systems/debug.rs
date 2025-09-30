#![allow(clippy::too_many_arguments)]
use crate::bundles::PlayerPhysicsBundle;
use crate::components::{ActiveEntity, MainCamera, Player};
use crate::game_state::GameState;
use bevy::prelude::*;
// Legacy input removed - use raw F1 for debug toggle

pub fn debug_game_state(
    current_state: Res<State<GameState>>,
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut state: ResMut<NextState<GameState>>,
    player_query: Query<Entity, With<Player>>,
    active_player_query: Query<Entity, (With<Player>, With<ActiveEntity>)>,
    active_any_query: Query<Entity, With<ActiveEntity>>,
    camera_query: Query<Entity, With<MainCamera>>,
) {
    if input.just_pressed(KeyCode::F1) {
        info!("=== DEBUG INFO ===");
        info!("Current game state: {:?}", **current_state);
        info!("Players found: {}", player_query.iter().count());
        info!(
            "Active players found: {}",
            active_player_query.iter().count()
        );
        info!(
            "Any active entities found: {}",
            active_any_query.iter().count()
        );
        info!("Cameras found: {}", camera_query.iter().count());

        // List all active entities
        for entity in active_any_query.iter() {
            info!("Active entity: {:?}", entity);
        }

        if let Some(any_input) = [
            KeyCode::ArrowUp,
            KeyCode::ArrowDown,
            KeyCode::ArrowLeft,
            KeyCode::ArrowRight,
        ]
        .iter()
        .find(|key| input.pressed(**key))
        {
            info!("Arrow key pressed: {:?}", any_input);
        }
    }

    // Emergency fix: F2 ONLY to force restore player ActiveEntity and set to Walking + reset input system
    if input.just_pressed(KeyCode::F2) {
        info!("=== EMERGENCY RESET ===");

        // Reset player state
        if let Ok(player_entity) = player_query.single() {
            commands
                .entity(player_entity)
                .insert(ActiveEntity)
                .insert(Visibility::Visible)
                .remove::<ChildOf>()
                .insert(PlayerPhysicsBundle::default()); // Restore clean physics state
            state.set(GameState::Walking);
            info!("Restored player ActiveEntity and set to Walking state");
        } else {
            warn!("No player entity found to fix!");
        }

        // Reset input system
        // Legacy input managers removed - emergency reset simplified
        info!("Emergency reset completed");
    }
}
