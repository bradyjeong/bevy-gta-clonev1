use bevy::prelude::*;
use crate::components::ControlsText;
use crate::game_state::GameState;
use crate::systems::input::{LoadedVehicleControls, get_vehicle_control_help};
use crate::components::VehicleControlType;

pub fn controls_ui_system(
    current_state: Res<State<GameState>>,
    loaded_controls: Res<LoadedVehicleControls>,
    mut controls_query: Query<&mut Text, With<ControlsText>>,
) {
    for mut text in controls_query.iter_mut() {
        let controls_text = generate_dynamic_controls_text(current_state.get(), &loaded_controls);
        text.0 = controls_text;
    }
}

fn generate_dynamic_controls_text(state: &GameState, loaded_controls: &LoadedVehicleControls) -> String {
    // Convert GameState to VehicleControlType
    let vehicle_type = match state {
        GameState::Walking => VehicleControlType::Walking,
        GameState::Driving => VehicleControlType::Car,
        GameState::Flying => VehicleControlType::Helicopter,
        GameState::Jetting => VehicleControlType::F16,
    };
    
    // Use asset-based control help generation
    if let Some(help_text) = get_vehicle_control_help(&vehicle_type, loaded_controls) {
        help_text
    } else {
        // Fallback if controls haven't loaded yet
        match state {
            GameState::Walking => "LOADING WALKING CONTROLS...".to_string(),
            GameState::Driving => "LOADING VEHICLE CONTROLS...".to_string(),
            GameState::Flying => "LOADING HELICOPTER CONTROLS...".to_string(),
            GameState::Jetting => "LOADING F16 CONTROLS...".to_string(),
        }
    }
}

// Remove the old hardcoded UI generation - now using asset-based system

#[allow(dead_code)]
fn format_key_name(key: KeyCode) -> String {
    match key {
        KeyCode::ArrowUp => "⬆️ UP".to_string(),
        KeyCode::ArrowDown => "⬇️ DOWN".to_string(),
        KeyCode::ArrowLeft => "⬅️ LEFT".to_string(),
        KeyCode::ArrowRight => "➡️ RIGHT".to_string(),
        KeyCode::Space => "🚀 SPACE".to_string(),
        KeyCode::ShiftLeft => "🚀 SHIFT".to_string(),
        KeyCode::ControlLeft => "CTRL".to_string(),
        KeyCode::KeyF => "🔄 F".to_string(),
        KeyCode::KeyQ => "Q".to_string(),
        KeyCode::KeyE => "E".to_string(),
        KeyCode::KeyW => "🔥 W".to_string(),
        KeyCode::KeyA => "↪️ A".to_string(),
        KeyCode::KeyS => "🔻 S".to_string(),
        KeyCode::KeyD => "↩️ D".to_string(),
        KeyCode::Enter => "🔄 ENTER".to_string(),
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
