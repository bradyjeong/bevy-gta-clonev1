//! ───────────────────────────────────────────────
//! System:   Controls Ui
//! Purpose:  Handles entity movement and physics
//! Schedule: Update
//! Reads:    InputConfig, ControlsText, mut
//! Writes:   Text
//! Invariants:
//!   * System maintains consistent state
//! Owner:    @ui-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use game_core::prelude::*;
use gameplay_sim::systems::input::{InputConfig, InputAction};

/// Controls UI system
pub fn controls_ui_system(
    current_state: Res<State<GameState>>,
    input_config: Res<InputConfig>,
    mut controls_query: Query<&mut Text, With<ControlsText>>,
) {
    for mut text in controls_query.iter_mut() {
        let controls_text = generate_dynamic_controls_text(current_state.clone(), &input_config);
        text.0 = controls_text;
    }
}

fn generate_dynamic_controls_text(state: GameState, input_config: &InputConfig) -> String {
    match state {
        GameState::Walking => {
            let mut controls = Vec::new();
            controls.push("CONTROLS - Walking:\n".to_string());
            
            // Movement controls
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::Forward) {
                controls.push(format!("{}: Forward", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::Backward) {
                controls.push(format!("{}: Backward", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::TurnLeft) {
                controls.push(format!("{}: Turn Left", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::TurnRight) {
                controls.push(format!("{}: Turn Right", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::Run) {
                controls.push(format!("{}: Run", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::Interact) {
                controls.push(format!("{}: Enter Vehicle", format_key_name(key)));
            }
            controls.join("\n")
        },
        
        GameState::Driving => {
            let mut controls = Vec::new();
            controls.push("CONTROLS - Car/SuperCar:\n".to_string());
            
            // Driving controls
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::Forward) {
                controls.push(format!("{}: Accelerate", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::Backward) {
                controls.push(format!("{}: Brake/Reverse", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::TurnLeft) {
                controls.push(format!("{}: Turn Left", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::TurnRight) {
                controls.push(format!("{}: Turn Right", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::Interact) {
                controls.push(format!("{}: Exit Vehicle", format_key_name(key)));
            }
            controls.join("\n")
        },
        
        GameState::Flying => {
            let mut controls = Vec::new();
            controls.push("CONTROLS - Aircraft:\n".to_string());
            
            // Flight controls
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::Forward) {
                controls.push(format!("{}: Pitch Down", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::Backward) {
                controls.push(format!("{}: Pitch Up", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::TurnLeft) {
                controls.push(format!("{}: Roll Left", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::TurnRight) {
                controls.push(format!("{}: Roll Right", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::YawLeft) {
                controls.push(format!("{}: Yaw Left", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::YawRight) {
                controls.push(format!("{}: Yaw Right", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::Forward) {
                controls.push(format!("{}: Throttle", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::Backward) {
                controls.push(format!("{}: Brake", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::Interact) {
                controls.push(format!("{}: Exit Aircraft", format_key_name(key)));
            }
            controls.join("\n")
        },
        
        _ => {
            "No controls available for this state".to_string()
        }
    }
}

fn format_key_name(key: KeyCode) -> String {
    match key {
        KeyCode::KeyQ => "Q".to_string(),
        KeyCode::KeyE => "E".to_string(),
        KeyCode::KeyW => "W".to_string(),
        KeyCode::KeyA => "A".to_string(),
        KeyCode::KeyS => "S".to_string(),
        KeyCode::KeyD => "D".to_string(),
        KeyCode::F1 => "F1".to_string(),
        KeyCode::F2 => "F2".to_string(),
        KeyCode::F3 => "F3".to_string(),
        KeyCode::F4 => "F4".to_string(),
        // Extract just the key part from debug format (removes "Key" prefix)
        _ => {
            let debug_str = format!("{:?}", key);
            if debug_str.starts_with("Key") {
                debug_str.strip_prefix("Key").unwrap_or(&debug_str).to_string()
            } else {
                debug_str
            }
        }
    }
}
