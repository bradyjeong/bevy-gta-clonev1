use bevy::prelude::*;
use crate::components::{Player, ActiveEntity, MainCamera};
use crate::game_state::GameState;
use crate::systems::input::{InputManager, InputConfig, InputAction};

pub fn debug_game_state(
    current_state: Res<State<GameState>>,
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut state: ResMut<NextState<GameState>>,
    player_query: Query<Entity, With<Player>>,
    active_player_query: Query<Entity, (With<Player>, With<ActiveEntity>)>,
    active_any_query: Query<Entity, With<ActiveEntity>>,
    camera_query: Query<Entity, With<MainCamera>>,
    mut input_manager: ResMut<InputManager>,
    mut input_config: ResMut<InputConfig>,
) {
    if input.just_pressed(KeyCode::F1) || input_manager.is_action_just_pressed(InputAction::DebugInfo) {
        info!("=== DEBUG INFO ===");
        info!("Current game state: {:?}", **current_state);
        info!("Players found: {}", player_query.iter().count());
        info!("Active players found: {}", active_player_query.iter().count());
        info!("Any active entities found: {}", active_any_query.iter().count());
        info!("Cameras found: {}", camera_query.iter().count());
        
        // Input system performance stats
        let (max_time_us, frame_count) = input_manager.get_performance_stats();
        info!("Input system - Max processing time: {}μs, Frames: {}", max_time_us, frame_count);
        info!("Input fallback enabled: {}", input_config.is_fallback_enabled());
        
        // List all active entities
        for entity in active_any_query.iter() {
            info!("Active entity: {:?}", entity);
        }
        
        // Show active input actions
        let active_actions = input_manager.get_active_actions();
        if !active_actions.is_empty() {
            info!("Active input actions: {:?}", active_actions);
        }
        
        if let Some(any_input) = [
            KeyCode::ArrowUp, KeyCode::ArrowDown, 
            KeyCode::ArrowLeft, KeyCode::ArrowRight
        ].iter().find(|key| input.pressed(**key)) {
            info!("Arrow key pressed: {:?}", any_input);
        }
    }
    
    // Emergency fix: F2 ONLY to force restore player ActiveEntity and set to Walking + reset input system
    if input.just_pressed(KeyCode::F2) {
        info!("=== EMERGENCY RESET ===");
        
        // Reset player state
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
        
        // Reset input system
        input_manager.clear_all_input();
        input_config.reset_to_defaults();
        input_config.enable_fallback(); // Enable fallback mode for safety
        input_manager.reset_performance_stats();
        info!("Reset input system to defaults with fallback enabled");
    }
}
