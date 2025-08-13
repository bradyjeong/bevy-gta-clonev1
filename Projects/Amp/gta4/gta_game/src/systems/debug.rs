use bevy::prelude::*;
use crate::components::{Player, ActiveEntity, MainCamera};
use crate::game_state::GameState;

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
        info!("Active players found: {}", active_player_query.iter().count());
        info!("Any active entities found: {}", active_any_query.iter().count());
        info!("Cameras found: {}", camera_query.iter().count());
        
        // List all active entities
        for entity in active_any_query.iter() {
            info!("Active entity: {:?}", entity);
        }
        
        if let Some(any_input) = [
            KeyCode::ArrowUp, KeyCode::ArrowDown, 
            KeyCode::ArrowLeft, KeyCode::ArrowRight
        ].iter().find(|key| input.pressed(**key)) {
            info!("Arrow key pressed: {:?}", any_input);
        }
    }
    
    // Emergency fix: F2 to force restore player ActiveEntity and set to Walking
    if input.just_pressed(KeyCode::F2) {
        info!("=== EMERGENCY FIX ===");
        if let Ok(player_entity) = player_query.single() {
            commands.entity(player_entity)
                .insert(ActiveEntity)
                .insert(Visibility::Visible)
                .remove::<ChildOf>();
            state.set(GameState::Walking);
            info!("Restored player ActiveEntity and set to Walking state");
        } else {
            warn!("No player entity found to fix!");
        }
    }
}
