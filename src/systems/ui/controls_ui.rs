use crate::components::{ActiveEntity, ControlsText, VehicleControlType};
use crate::game_state::GameState;
use crate::systems::input::{get_vehicle_control_help, LoadedVehicleControls};
use bevy::prelude::*;

pub fn controls_ui_system(
    current_state: Res<State<GameState>>,
    loaded_controls: Res<LoadedVehicleControls>,
    mut controls_query: Query<&mut Text, With<ControlsText>>,
    active_vehicle_query: Query<&VehicleControlType, With<ActiveEntity>>,
) {
    for mut text in controls_query.iter_mut() {
        let controls_text = generate_dynamic_controls_text(
            current_state.get(),
            &loaded_controls,
            &active_vehicle_query,
        );
        text.0 = controls_text;
    }
}

fn generate_dynamic_controls_text(
    state: &GameState,
    loaded_controls: &LoadedVehicleControls,
    active_vehicle_query: &Query<&VehicleControlType, With<ActiveEntity>>,
) -> String {
    // Get actual VehicleControlType from active entity if available
    let vehicle_type = if let Ok(vehicle_control_type) = active_vehicle_query.single() {
        *vehicle_control_type
    } else {
        // Fallback to GameState-based detection
        match state {
            GameState::Walking => VehicleControlType::Walking,
            GameState::Swimming => VehicleControlType::Swimming,
            GameState::Driving => VehicleControlType::Car,
            GameState::Flying => VehicleControlType::Helicopter,
            GameState::Jetting => VehicleControlType::F16,
        }
    };

    // Use asset-based control help generation
    if let Some(help_text) = get_vehicle_control_help(&vehicle_type, loaded_controls) {
        help_text
    } else {
        // Fallback if controls haven't loaded yet
        match vehicle_type {
            VehicleControlType::Walking => "LOADING WALKING CONTROLS...".to_string(),
            VehicleControlType::Swimming => "LOADING SWIMMING CONTROLS...".to_string(),
            VehicleControlType::Car => "LOADING CAR CONTROLS...".to_string(),
            VehicleControlType::Helicopter => "LOADING HELICOPTER CONTROLS...".to_string(),
            VehicleControlType::F16 => "LOADING F16 CONTROLS...".to_string(),
            VehicleControlType::Yacht => "LOADING YACHT CONTROLS...".to_string(),
        }
    }
}

// Remove the old hardcoded UI generation - now using asset-based system

#[allow(dead_code)]
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
        KeyCode::Enter => "ENTER".to_string(),
        KeyCode::F1 => "F1".to_string(),
        KeyCode::F2 => "F2".to_string(),
        KeyCode::F3 => "F3".to_string(),
        KeyCode::F4 => "F4".to_string(),
        // Extract just the key part from debug format (removes "Key" prefix)
        _ => {
            let debug_str = format!("{key:?}");
            if debug_str.starts_with("Key") {
                debug_str
                    .strip_prefix("Key")
                    .unwrap_or(&debug_str)
                    .to_string()
            } else {
                debug_str
            }
        }
    }
}
