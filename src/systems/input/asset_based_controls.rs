use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::components::{ControlState, VehicleControlType};

/// Asset-based control configuration system
/// 
/// This system loads control mappings from RON files instead of hardcoding them.
/// Benefits:
/// - No code changes needed for new vehicles or control schemes
/// - Easy to customize controls without recompilation
/// - Single source of truth for all control mappings
/// - Supports runtime control customization

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssetControlAction {
    // Movement actions
    Forward,
    Backward,
    TurnLeft,
    TurnRight,
    
    // Flight actions
    PitchUp,
    PitchDown,
    PitchForward,
    PitchBackward,
    RollLeft,
    RollRight,
    YawLeft,
    YawRight,
    
    // Vertical actions
    VerticalUp,
    VerticalDown,
    
    // Power actions
    ThrottleUp,
    ThrottleDown,
    Turbo,
    Afterburner,
    
    // Meta actions
    Run,
    Interact,
    DebugInfo,
    EmergencyReset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetControlBinding {
    pub action: AssetControlAction,
    pub key: KeyCode,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleControls {
    pub name: String,
    pub description: String,
    pub primary_controls: Vec<AssetControlBinding>,
    pub secondary_controls: Vec<AssetControlBinding>,
    pub meta_controls: Vec<AssetControlBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
pub struct VehicleControlsConfig {
    pub vehicle_types: HashMap<VehicleControlType, VehicleControls>,
}

impl VehicleControls {
    /// Get all control bindings for this vehicle
    pub fn get_all_bindings(&self) -> Vec<&AssetControlBinding> {
        let mut bindings = Vec::new();
        bindings.extend(&self.primary_controls);
        bindings.extend(&self.secondary_controls);
        bindings.extend(&self.meta_controls);
        bindings
    }
    
    /// Get key for a specific action
    pub fn get_key_for_action(&self, action: &AssetControlAction) -> Option<KeyCode> {
        self.get_all_bindings()
            .iter()
            .find(|binding| std::mem::discriminant(&binding.action) == std::mem::discriminant(action))
            .map(|binding| binding.key)
    }
    
    /// Check if an action is bound for this vehicle
    pub fn has_action(&self, action: &AssetControlAction) -> bool {
        self.get_key_for_action(action).is_some()
    }
}

#[derive(Resource, Default)]
pub struct LoadedVehicleControls {
    pub config: Option<VehicleControlsConfig>,
    pub loading: bool,
}

#[derive(Resource)]
pub struct VehicleControlsHandle(pub Handle<VehicleControlsConfig>);

/// System to load vehicle controls from assets
pub fn load_vehicle_controls_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loaded_controls: ResMut<LoadedVehicleControls>,
) {
    if loaded_controls.config.is_none() && !loaded_controls.loading {
        info!("Loading vehicle controls from assets/config/vehicle_controls.ron");
        let handle: Handle<VehicleControlsConfig> = asset_server.load("config/vehicle_controls.ron");
        commands.insert_resource(VehicleControlsHandle(handle));
        loaded_controls.loading = true;
    }
}

/// System to process loaded control assets
pub fn process_loaded_controls_system(
    mut loaded_controls: ResMut<LoadedVehicleControls>,
    controls_assets: Res<Assets<VehicleControlsConfig>>,
    controls_handle: Option<Res<VehicleControlsHandle>>,
) {
    if let Some(handle) = controls_handle {
        if let Some(config) = controls_assets.get(&handle.0) {
            if loaded_controls.config.is_none() {
                info!("Vehicle controls loaded successfully!");
                loaded_controls.config = Some(config.clone());
                loaded_controls.loading = false;
            }
        }
    }
}

/// Asset-based input mapping system
/// 
/// This system uses loaded control configurations instead of hardcoded mappings
/// Only processes entities with ActiveEntity to prevent state conflicts
pub fn asset_based_input_mapping_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    loaded_controls: Res<LoadedVehicleControls>,
    mut query: Query<(&mut ControlState, &VehicleControlType), With<crate::components::ActiveEntity>>,
) {
    // Always reset control state each frame, even if assets aren't loaded yet
    for (mut control_state, _vehicle_type) in query.iter_mut() {
        control_state.reset();
    }
    
    // Skip input mapping if controls haven't loaded yet
    let Some(ref config) = loaded_controls.config else {
        return;
    };
    
    for (mut control_state, vehicle_type) in query.iter_mut() {
        
        // Get vehicle controls from loaded config
        let Some(vehicle_controls) = config.vehicle_types.get(vehicle_type) else {
            warn!("No controls found for vehicle type: {:?}", vehicle_type);
            continue;
        };
        
        // Map input based on loaded configuration
        for binding in vehicle_controls.get_all_bindings() {
            if keyboard_input.pressed(binding.key) {
                apply_control_action(&binding.action, &mut control_state);
            }
            
            // Handle just_pressed actions
            if keyboard_input.just_pressed(binding.key) {
                apply_control_action_once(&binding.action, &mut control_state);
            }
        }
        
        // Always validate inputs for safety
        control_state.validate_and_clamp();
    }
}

/// Apply a control action to the control state (for held keys)
fn apply_control_action(action: &AssetControlAction, control_state: &mut ControlState) {
    match action {
        AssetControlAction::Forward => control_state.throttle = 1.0,
        AssetControlAction::Backward => control_state.brake = 1.0,
        AssetControlAction::TurnLeft => control_state.steering = 1.0,  // Turn left = positive rotation
        AssetControlAction::TurnRight => control_state.steering = -1.0, // Turn right = negative rotation
        
        AssetControlAction::PitchUp | AssetControlAction::PitchForward => control_state.pitch = 1.0,
        AssetControlAction::PitchDown | AssetControlAction::PitchBackward => control_state.pitch = -1.0,
        AssetControlAction::RollLeft => control_state.roll = -1.0,
        AssetControlAction::RollRight => control_state.roll = 1.0,
        AssetControlAction::YawLeft => control_state.yaw = -1.0,  // Yaw left = negative rotation (follows control_state.rs docs)
        AssetControlAction::YawRight => control_state.yaw = 1.0,  // Yaw right = positive rotation
        
        AssetControlAction::VerticalUp => control_state.vertical = 1.0,
        AssetControlAction::VerticalDown => control_state.vertical = -1.0,
        
        AssetControlAction::ThrottleUp => control_state.throttle = 1.0,
        AssetControlAction::ThrottleDown => control_state.brake = 1.0,
        AssetControlAction::Turbo | AssetControlAction::Afterburner => control_state.boost = 1.0,
        
        AssetControlAction::Run => control_state.run = true,
        
        // Meta actions are handled in apply_control_action_once
        _ => {}
    }
}

/// Apply a control action once (for just_pressed keys)
fn apply_control_action_once(action: &AssetControlAction, control_state: &mut ControlState) {
    match action {
        AssetControlAction::Interact => control_state.interact = true,
        AssetControlAction::EmergencyReset => control_state.emergency_brake = true,
        // Other actions are continuous, not one-shot
        _ => {}
    }
}

/// Helper function to get control help text from loaded config
pub fn get_vehicle_control_help(
    vehicle_type: &VehicleControlType,
    loaded_controls: &LoadedVehicleControls,
) -> Option<String> {
    let config = loaded_controls.config.as_ref()?;
    let vehicle_controls = config.vehicle_types.get(vehicle_type)?;
    
    let mut help_text = Vec::new();
    help_text.push(format!("{} CONTROLS", vehicle_controls.name.to_uppercase()));
    help_text.push(format!("{}\n", vehicle_controls.description));
    
    if !vehicle_controls.primary_controls.is_empty() {
        help_text.push("PRIMARY CONTROLS:".to_string());
        for binding in &vehicle_controls.primary_controls {
            help_text.push(format!("  {:?}: {}", binding.key, binding.description));
        }
        help_text.push("".to_string());
    }
    
    if !vehicle_controls.secondary_controls.is_empty() {
        help_text.push("SECONDARY CONTROLS:".to_string());
        for binding in &vehicle_controls.secondary_controls {
            help_text.push(format!("  {:?}: {}", binding.key, binding.description));
        }
        help_text.push("".to_string());
    }
    
    if !vehicle_controls.meta_controls.is_empty() {
        help_text.push("META CONTROLS:".to_string());
        for binding in &vehicle_controls.meta_controls {
            help_text.push(format!("  {:?}: {}", binding.key, binding.description));
        }
    }
    
    Some(help_text.join("\n"))
}

/// Debug system to display loaded control configuration
pub fn debug_loaded_controls_system(
    loaded_controls: Res<LoadedVehicleControls>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if !keyboard_input.just_pressed(KeyCode::F3) {
        return;
    }
    
    if let Some(ref config) = loaded_controls.config {
        info!("=== LOADED VEHICLE CONTROLS ===");
        for (vehicle_type, controls) in &config.vehicle_types {
            info!("{:?}: {} controls", vehicle_type, controls.get_all_bindings().len());
        }
    } else {
        info!("Vehicle controls not yet loaded");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_control_action_application() {
        let mut control_state = ControlState::default();
        
        apply_control_action(&AssetControlAction::Forward, &mut control_state);
        assert_eq!(control_state.throttle, 1.0);
        
        apply_control_action(&AssetControlAction::TurnLeft, &mut control_state);
        assert_eq!(control_state.steering, 1.0); // TurnLeft now maps to positive rotation
        
        apply_control_action_once(&AssetControlAction::Interact, &mut control_state);
        assert!(control_state.interact);
    }
    
    #[test]
    fn test_vehicle_controls_lookup() {
        let controls = VehicleControls {
            name: "Test".to_string(),
            description: "Test vehicle".to_string(),
            primary_controls: vec![
                AssetControlBinding {
                    action: AssetControlAction::Forward,
                    key: KeyCode::ArrowUp,
                    description: "Move forward".to_string(),
                }
            ],
            secondary_controls: vec![],
            meta_controls: vec![],
        };
        
        assert_eq!(controls.get_key_for_action(&AssetControlAction::Forward), Some(KeyCode::ArrowUp));
        assert_eq!(controls.get_key_for_action(&AssetControlAction::Backward), None);
        assert!(controls.has_action(&AssetControlAction::Forward));
        assert!(!controls.has_action(&AssetControlAction::Backward));
    }
}
