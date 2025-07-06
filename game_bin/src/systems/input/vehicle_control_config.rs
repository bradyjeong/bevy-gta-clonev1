use bevy::prelude::*;
use std::collections::HashMap;
use crate::game_state::GameState;
use super::input_config::InputAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VehicleType {
    Walking,
    Car,
    SuperCar,
    Helicopter,
    F16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlCategory {
    Primary,    // Movement, steering
    Secondary,  // Turbo, afterburner, vertical movement
    Meta,       // Exit vehicle, debug
}

#[derive(Debug, Clone)]
pub struct ControlBinding {
    pub action: InputAction,
    pub key: KeyCode,
    pub description: String,
    pub category: ControlCategory,
}

#[derive(Debug, Clone, Resource)]
pub struct VehicleControlConfig {
    // Vehicle-specific control mappings
    vehicle_controls: HashMap<VehicleType, Vec<ControlBinding>>,
    // Quick lookup: Vehicle -> Action -> KeyCode
    lookup_cache: HashMap<VehicleType, HashMap<InputAction, KeyCode>>,
}

impl Default for VehicleControlConfig {
    fn default() -> Self {
        let mut config = VehicleControlConfig {
            vehicle_controls: HashMap::new(),
            lookup_cache: HashMap::new(),
        };
        
        config.setup_default_controls();
        config.rebuild_lookup_cache();
        config
    }
}

impl VehicleControlConfig {
    fn setup_default_controls(&mut self) {
        // Walking controls
        self.vehicle_controls.insert(VehicleType::Walking, vec![
            // Primary controls
            ControlBinding {
                action: InputAction::Forward,
                key: KeyCode::ArrowUp,
                description: "Walk forward".to_string(),
                category: ControlCategory::Primary,
            },
            ControlBinding {
                action: InputAction::Backward,
                key: KeyCode::ArrowDown,
                description: "Walk backward".to_string(),
                category: ControlCategory::Primary,
            },
            ControlBinding {
                action: InputAction::TurnLeft,
                key: KeyCode::ArrowLeft,
                description: "Turn left".to_string(),
                category: ControlCategory::Primary,
            },
            ControlBinding {
                action: InputAction::TurnRight,
                key: KeyCode::ArrowRight,
                description: "Turn right".to_string(),
                category: ControlCategory::Primary,
            },
            // Secondary controls
            ControlBinding {
                action: InputAction::Run,
                key: KeyCode::ShiftLeft,
                description: "Run / Sprint".to_string(),
                category: ControlCategory::Secondary,
            },
            // Meta controls
            ControlBinding {
                action: InputAction::Interact,
                key: KeyCode::KeyF,
                description: "Enter vehicle / Interact".to_string(),
                category: ControlCategory::Meta,
            },
            ControlBinding {
                action: InputAction::DebugInfo,
                key: KeyCode::F1,
                description: "Toggle debug info".to_string(),
                category: ControlCategory::Meta,
            },
            ControlBinding {
                action: InputAction::EmergencyReset,
                key: KeyCode::F2,
                description: "Emergency reset".to_string(),
                category: ControlCategory::Meta,
            },
        ]);
        
        // Car controls
        self.vehicle_controls.insert(VehicleType::Car, vec![
            // Primary controls
            ControlBinding {
                action: InputAction::Forward,
                key: KeyCode::ArrowUp,
                description: "Accelerate / Drive forward".to_string(),
                category: ControlCategory::Primary,
            },
            ControlBinding {
                action: InputAction::Backward,
                key: KeyCode::ArrowDown,
                description: "Brake / Reverse".to_string(),
                category: ControlCategory::Primary,
            },
            ControlBinding {
                action: InputAction::TurnLeft,
                key: KeyCode::ArrowLeft,
                description: "Steer left".to_string(),
                category: ControlCategory::Primary,
            },
            ControlBinding {
                action: InputAction::TurnRight,
                key: KeyCode::ArrowRight,
                description: "Steer right".to_string(),
                category: ControlCategory::Primary,
            },
            // Secondary controls
            ControlBinding {
                action: InputAction::Turbo,
                key: KeyCode::Space,
                description: "Turbo boost".to_string(),
                category: ControlCategory::Secondary,
            },
            // Meta controls
            ControlBinding {
                action: InputAction::Interact,
                key: KeyCode::KeyF,
                description: "Exit vehicle".to_string(),
                category: ControlCategory::Meta,
            },
            ControlBinding {
                action: InputAction::DebugInfo,
                key: KeyCode::F1,
                description: "Toggle debug info".to_string(),
                category: ControlCategory::Meta,
            },
            ControlBinding {
                action: InputAction::EmergencyReset,
                key: KeyCode::F2,
                description: "Emergency reset".to_string(),
                category: ControlCategory::Meta,
            },
        ]);
        
        // SuperCar controls (same as Car but with enhanced descriptions)
        self.vehicle_controls.insert(VehicleType::SuperCar, vec![
            // Primary controls
            ControlBinding {
                action: InputAction::Forward,
                key: KeyCode::ArrowUp,
                description: "Accelerate (High Performance)".to_string(),
                category: ControlCategory::Primary,
            },
            ControlBinding {
                action: InputAction::Backward,
                key: KeyCode::ArrowDown,
                description: "Brake / Reverse (Racing Brakes)".to_string(),
                category: ControlCategory::Primary,
            },
            ControlBinding {
                action: InputAction::TurnLeft,
                key: KeyCode::ArrowLeft,
                description: "Steer left (Precision Steering)".to_string(),
                category: ControlCategory::Primary,
            },
            ControlBinding {
                action: InputAction::TurnRight,
                key: KeyCode::ArrowRight,
                description: "Steer right (Precision Steering)".to_string(),
                category: ControlCategory::Primary,
            },
            // Secondary controls
            ControlBinding {
                action: InputAction::Turbo,
                key: KeyCode::Space,
                description: "Nitrous Boost".to_string(),
                category: ControlCategory::Secondary,
            },
            // Meta controls
            ControlBinding {
                action: InputAction::Interact,
                key: KeyCode::KeyF,
                description: "Exit vehicle".to_string(),
                category: ControlCategory::Meta,
            },
            ControlBinding {
                action: InputAction::DebugInfo,
                key: KeyCode::F1,
                description: "Toggle debug info".to_string(),
                category: ControlCategory::Meta,
            },
            ControlBinding {
                action: InputAction::EmergencyReset,
                key: KeyCode::F2,
                description: "Emergency reset".to_string(),
                category: ControlCategory::Meta,
            },
        ]);
        
        // Helicopter controls
        self.vehicle_controls.insert(VehicleType::Helicopter, vec![
            // Primary controls
            ControlBinding {
                action: InputAction::Forward,
                key: KeyCode::ArrowUp,
                description: "Pitch forward / Move forward".to_string(),
                category: ControlCategory::Primary,
            },
            ControlBinding {
                action: InputAction::Backward,
                key: KeyCode::ArrowDown,
                description: "Pitch backward / Move backward".to_string(),
                category: ControlCategory::Primary,
            },
            ControlBinding {
                action: InputAction::TurnLeft,
                key: KeyCode::ArrowLeft,
                description: "Yaw left / Turn left".to_string(),
                category: ControlCategory::Primary,
            },
            ControlBinding {
                action: InputAction::TurnRight,
                key: KeyCode::ArrowRight,
                description: "Yaw right / Turn right".to_string(),
                category: ControlCategory::Primary,
            },
            // Secondary controls
            ControlBinding {
                action: InputAction::VerticalUp,
                key: KeyCode::ShiftLeft,
                description: "Collective up / Ascend".to_string(),
                category: ControlCategory::Secondary,
            },
            ControlBinding {
                action: InputAction::VerticalDown,
                key: KeyCode::ControlLeft,
                description: "Collective down / Descend".to_string(),
                category: ControlCategory::Secondary,
            },
            // Meta controls
            ControlBinding {
                action: InputAction::Interact,
                key: KeyCode::KeyF,
                description: "Exit vehicle".to_string(),
                category: ControlCategory::Meta,
            },
            ControlBinding {
                action: InputAction::DebugInfo,
                key: KeyCode::F1,
                description: "Toggle debug info".to_string(),
                category: ControlCategory::Meta,
            },
            ControlBinding {
                action: InputAction::EmergencyReset,
                key: KeyCode::F2,
                description: "Emergency reset".to_string(),
                category: ControlCategory::Meta,
            },
        ]);
        
        // F16 controls
        self.vehicle_controls.insert(VehicleType::F16, vec![
            // Primary controls (Flight stick)
            ControlBinding {
                action: InputAction::PitchUp,
                key: KeyCode::KeyW,
                description: "Pitch up (Nose up)".to_string(),
                category: ControlCategory::Primary,
            },
            ControlBinding {
                action: InputAction::PitchDown,
                key: KeyCode::KeyS,
                description: "Pitch down (Nose down)".to_string(),
                category: ControlCategory::Primary,
            },
            ControlBinding {
                action: InputAction::RollLeft,
                key: KeyCode::KeyA,
                description: "Roll left".to_string(),
                category: ControlCategory::Primary,
            },
            ControlBinding {
                action: InputAction::RollRight,
                key: KeyCode::KeyD,
                description: "Roll right".to_string(),
                category: ControlCategory::Primary,
            },
            ControlBinding {
                action: InputAction::YawLeft,
                key: KeyCode::KeyQ,
                description: "Rudder left / Yaw left".to_string(),
                category: ControlCategory::Primary,
            },
            ControlBinding {
                action: InputAction::YawRight,
                key: KeyCode::KeyE,
                description: "Rudder right / Yaw right".to_string(),
                category: ControlCategory::Primary,
            },
            // Primary controls (Alternative arrow keys)
            ControlBinding {
                action: InputAction::Forward,
                key: KeyCode::ArrowUp,
                description: "Thrust forward (Alt)".to_string(),
                category: ControlCategory::Primary,
            },
            ControlBinding {
                action: InputAction::Backward,
                key: KeyCode::ArrowDown,
                description: "Thrust backward (Alt)".to_string(),
                category: ControlCategory::Primary,
            },
            ControlBinding {
                action: InputAction::TurnLeft,
                key: KeyCode::ArrowLeft,
                description: "Turn left (Alt)".to_string(),
                category: ControlCategory::Primary,
            },
            ControlBinding {
                action: InputAction::TurnRight,
                key: KeyCode::ArrowRight,
                description: "Turn right (Alt)".to_string(),
                category: ControlCategory::Primary,
            },
            // Secondary controls
            ControlBinding {
                action: InputAction::VerticalUp,
                key: KeyCode::ShiftLeft,
                description: "Throttle up / Climb".to_string(),
                category: ControlCategory::Secondary,
            },
            ControlBinding {
                action: InputAction::VerticalDown,
                key: KeyCode::ControlLeft,
                description: "Throttle down / Dive".to_string(),
                category: ControlCategory::Secondary,
            },
            ControlBinding {
                action: InputAction::Afterburner,
                key: KeyCode::Space,
                description: "Afterburner / Max thrust".to_string(),
                category: ControlCategory::Secondary,
            },
            // Meta controls
            ControlBinding {
                action: InputAction::Interact,
                key: KeyCode::KeyF,
                description: "Exit vehicle".to_string(),
                category: ControlCategory::Meta,
            },
            ControlBinding {
                action: InputAction::DebugInfo,
                key: KeyCode::F1,
                description: "Toggle debug info".to_string(),
                category: ControlCategory::Meta,
            },
            ControlBinding {
                action: InputAction::EmergencyReset,
                key: KeyCode::F2,
                description: "Emergency reset".to_string(),
                category: ControlCategory::Meta,
            },
        ]);
    }
    
    fn rebuild_lookup_cache(&mut self) {
        self.lookup_cache.clear();
        
        for (vehicle_type, controls) in &self.vehicle_controls {
            let mut vehicle_lookup = HashMap::new();
            for control in controls {
                vehicle_lookup.insert(control.action, control.key);
            }
            self.lookup_cache.insert(*vehicle_type, vehicle_lookup);
        }
    }
    
    /// Get the key for a specific action on a vehicle type
    pub fn get_key_for_vehicle_action(&self, vehicle_type: VehicleType, action: InputAction) -> Option<KeyCode> {
        self.lookup_cache
            .get(&vehicle_type)
            .and_then(|vehicle_bindings| vehicle_bindings.get(&action))
            .copied()
    }
    
    /// Get all controls for a specific vehicle type
    pub fn get_vehicle_controls(&self, vehicle_type: VehicleType) -> Option<&Vec<ControlBinding>> {
        self.vehicle_controls.get(&vehicle_type)
    }
    
    /// Get controls filtered by category
    pub fn get_vehicle_controls_by_category(&self, vehicle_type: VehicleType, category: ControlCategory) -> Vec<&ControlBinding> {
        self.vehicle_controls
            .get(&vehicle_type)
            .map(|controls| {
                controls.iter()
                    .filter(|control| control.category == category)
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Get all available actions for a vehicle type
    pub fn get_available_actions(&self, vehicle_type: VehicleType) -> Vec<InputAction> {
        self.vehicle_controls
            .get(&vehicle_type)
            .map(|controls| controls.iter().map(|c| c.action).collect())
            .unwrap_or_default()
    }
    
    /// Check if an action is available for a vehicle type
    pub fn is_action_available_for_vehicle(&self, vehicle_type: VehicleType, action: InputAction) -> bool {
        self.lookup_cache
            .get(&vehicle_type)
            .map(|bindings| bindings.contains_key(&action))
            .unwrap_or(false)
    }
    
    /// Convert GameState to VehicleType for compatibility
    pub fn game_state_to_vehicle_type(state: &GameState) -> VehicleType {
        match state {
            GameState::Walking => VehicleType::Walking,
            GameState::Driving => VehicleType::Car, // Default to Car, could be enhanced to detect SuperCar
            GameState::Flying => VehicleType::Helicopter,
            GameState::Jetting => VehicleType::F16,
        }
    }
    
    /// Get controls for current game state
    pub fn get_controls_for_state(&self, state: &GameState) -> Option<&Vec<ControlBinding>> {
        let vehicle_type = Self::game_state_to_vehicle_type(state);
        self.get_vehicle_controls(vehicle_type)
    }
    
    /// Update a control binding for a vehicle type
    pub fn update_control(&mut self, vehicle_type: VehicleType, action: InputAction, new_key: KeyCode) -> Result<(), String> {
        // Check for conflicts within the same vehicle type
        if let Some(controls) = self.vehicle_controls.get(&vehicle_type) {
            for control in controls {
                if control.key == new_key && control.action != action {
                    return Err(format!("Key {:?} already bound to {:?} for {:?}", new_key, control.action, vehicle_type));
                }
            }
        }
        
        // Update the control
        if let Some(controls) = self.vehicle_controls.get_mut(&vehicle_type) {
            if let Some(control) = controls.iter_mut().find(|c| c.action == action) {
                control.key = new_key;
                self.rebuild_lookup_cache();
                return Ok(());
            }
        }
        
        Err(format!("Action {:?} not found for vehicle type {:?}", action, vehicle_type))
    }
    
    /// Add a new control binding to a vehicle type
    pub fn add_control(&mut self, vehicle_type: VehicleType, binding: ControlBinding) -> Result<(), String> {
        // Check for conflicts
        if let Some(controls) = self.vehicle_controls.get(&vehicle_type) {
            for control in controls {
                if control.key == binding.key {
                    return Err(format!("Key {:?} already bound to {:?} for {:?}", binding.key, control.action, vehicle_type));
                }
                if control.action == binding.action {
                    return Err(format!("Action {:?} already exists for {:?}", binding.action, vehicle_type));
                }
            }
        }
        
        // Add the control
        self.vehicle_controls
            .entry(vehicle_type)
            .or_insert_with(Vec::new)
            .push(binding);
        
        self.rebuild_lookup_cache();
        Ok(())
    }
    
    /// Generate UI-friendly control descriptions grouped by category
    pub fn get_control_help(&self, vehicle_type: VehicleType) -> HashMap<ControlCategory, Vec<String>> {
        let mut help_map = HashMap::new();
        
        if let Some(controls) = self.vehicle_controls.get(&vehicle_type) {
            for control in controls {
                help_map
                    .entry(control.category)
                    .or_insert_with(Vec::new)
                    .push(format!("{:?}: {}", control.key, control.description));
            }
        }
        
        help_map
    }
    
    /// Reset all controls to defaults
    pub fn reset_to_defaults(&mut self) {
        self.vehicle_controls.clear();
        self.setup_default_controls();
        self.rebuild_lookup_cache();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = VehicleControlConfig::default();
        
        // Test walking controls
        assert_eq!(
            config.get_key_for_vehicle_action(VehicleType::Walking, InputAction::Forward),
            Some(KeyCode::ArrowUp)
        );
        
        // Test car controls
        assert_eq!(
            config.get_key_for_vehicle_action(VehicleType::Car, InputAction::Turbo),
            Some(KeyCode::Space)
        );
        
        // Test F16 controls
        assert_eq!(
            config.get_key_for_vehicle_action(VehicleType::F16, InputAction::PitchUp),
            Some(KeyCode::KeyW)
        );
    }
    
    #[test]
    fn test_control_categories() {
        let config = VehicleControlConfig::default();
        
        let primary_controls = config.get_vehicle_controls_by_category(VehicleType::Car, ControlCategory::Primary);
        assert!(!primary_controls.is_empty());
        
        let meta_controls = config.get_vehicle_controls_by_category(VehicleType::Car, ControlCategory::Meta);
        assert!(!meta_controls.is_empty());
    }
    
    #[test]
    fn test_game_state_conversion() {
        assert_eq!(
            VehicleControlConfig::game_state_to_vehicle_type(&GameState::Walking),
            VehicleType::Walking
        );
        assert_eq!(
            VehicleControlConfig::game_state_to_vehicle_type(&GameState::Jetting),
            VehicleType::F16
        );
    }
    
    #[test]
    fn test_control_updates() {
        let mut config = VehicleControlConfig::default();
        
        // Valid update
        let result = config.update_control(VehicleType::Walking, InputAction::Forward, KeyCode::KeyW);
        assert!(result.is_ok());
        assert_eq!(
            config.get_key_for_vehicle_action(VehicleType::Walking, InputAction::Forward),
            Some(KeyCode::KeyW)
        );
        
        // Conflict detection
        let result = config.update_control(VehicleType::Walking, InputAction::Backward, KeyCode::KeyW);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_control_help_generation() {
        let config = VehicleControlConfig::default();
        let help = config.get_control_help(VehicleType::F16);
        
        assert!(help.contains_key(&ControlCategory::Primary));
        assert!(help.contains_key(&ControlCategory::Secondary));
        assert!(help.contains_key(&ControlCategory::Meta));
    }
}
