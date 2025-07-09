//! ───────────────────────────────────────────────
//! System:   Controls Ui
//! Purpose:  Handles entity movement and physics
//! Schedule: Update
//! Reads:    InputConfig, ControlsText, mut
//! Writes:   Text
//! Invariants:
//!   * System maintains consistent state
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use crate::components::ControlsText;
use gameplay_sim::prelude::GameState;
use crate::systems::input::input_config::{InputConfig, InputAction};

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
                controls.push(format!("{}: Reverse", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::TurnLeft) {
                controls.push(format!("{}: Steer Left", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::TurnRight) {
                controls.push(format!("{}: Steer Right", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::Turbo) {
                controls.push(format!("{}: Turbo Boost (SuperCar only)", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::Interact) {
                controls.push(format!("{}: Exit Car", format_key_name(key)));
            }
            
            controls.push("\nSPECIAL CONTROLS:".to_string());
            controls.push("F3: Performance Monitor".to_string());
            controls.push("F4: Bugatti Dashboard (SuperCar only)".to_string());
            controls.push("\nHold keys for acceleration".to_string());
            controls.push("Find the Bugatti Chiron for max speed!".to_string());
            
            controls.join("\n")
        },
        
        GameState::Flying => {
            let mut controls = Vec::new();
            controls.push("🚁 HELICOPTER CONTROLS\n".to_string());
            controls.push("🎮 FLIGHT CONTROLS:".to_string());
            
            // Flight controls
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::Forward) {
                controls.push(format!("▶️  {} - Forward", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::Backward) {
                controls.push(format!("◀️  {} - Backward", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::TurnLeft) {
                controls.push(format!("↖️  {} - Turn Left", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::TurnRight) {
                controls.push(format!("↗️  {} - Turn Right", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::VerticalUp) {
                controls.push(format!("⬆️  {} - Ascend / Gain Altitude", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::VerticalDown) {
                controls.push(format!("⬇️  {} - Descend / Lose Altitude", format_key_name(key)));
            }
            
            controls.push("\n🚪 EXIT:".to_string());
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::Interact) {
                controls.push(format!("🔄 {} - Exit Helicopter", format_key_name(key)));
            }
            
            controls.push("\n💡 TIP: Hold keys for smoother flight control".to_string());
            controls.push("🎯 Master the skies like a pro pilot!".to_string());
            
            controls.join("\n")
        },
        
        GameState::Jetting => {
            let mut controls = Vec::new();
            controls.push("CONTROLS - F16 Fighter:\n".to_string());
            
            // F16 controls
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
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::PitchUp) {
                controls.push(format!("{}: Pitch Up", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::PitchDown) {
                controls.push(format!("{}: Pitch Down", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::Afterburner) {
                controls.push(format!("{}: Afterburner", format_key_name(key)));
            }
            if let Some(key) = input_config.get_key_for_action(&state, InputAction::Interact) {
                controls.push(format!("{}: Exit F16", format_key_name(key)));
            }
            
            controls.join("\n")
        },
    }
}

fn format_key_name(key: KeyCode) -> String {
    match key {
        KeyCode::ArrowUp => "UP".to_string(),
        KeyCode::ArrowDown => "DOWN".to_string(),
        KeyCode::ArrowLeft => "LEFT".to_string(),
        KeyCode::ArrowRight => "RIGHT".to_string(),
        KeyCode::Space => "SPACE".to_string(),
        KeyCode::ShiftLeft => "SHIFT".to_string(),
        KeyCode::ControlLeft => "CTRL".to_string(),
        KeyCode::KeyF => "F".to_string(),
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
