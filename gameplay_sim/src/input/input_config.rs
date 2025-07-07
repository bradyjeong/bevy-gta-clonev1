//! ───────────────────────────────────────────────
//! System:   Input Config
//! Purpose:  Handles entity movement and physics
//! Schedule: Update
//! Reads:    System components
//! Writes:   System state
//! Invariants:
//!   * All values are validated for safety
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use std::collections::HashMap;
use game_core::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputAction {
    // Movement actions
    Forward,
    Backward,
    TurnLeft,
    TurnRight,
    
    // Vertical movement (helicopters/F16)
    VerticalUp,
    VerticalDown,
    
    // F16 specific
    PitchUp,
    PitchDown,
    RollLeft,
    RollRight,
    YawLeft,
    YawRight,
    
    // Modifiers
    Run,
    Turbo,
    Afterburner,
    
    // Interaction
    Interact,
    
    // Debug
    DebugInfo,
    EmergencyReset,
}

#[derive(Debug, Clone, Resource)]
pub struct InputConfig {
    // State-specific bindings
    bindings: HashMap<GameState, HashMap<InputAction, KeyCode>>,
    // Fallback to default bindings if custom fails
    use_fallback: bool,
}

impl Default for InputConfig {
    fn default() -> Self {
        let mut config = InputConfig {
            bindings: HashMap::new(),
            use_fallback: true, // Start with fallback enabled for safety
        };
        
        // Walking state bindings
        let mut walking_bindings = HashMap::new();
        walking_bindings.insert(InputAction::Forward, KeyCode::ArrowUp);
        walking_bindings.insert(InputAction::Backward, KeyCode::ArrowDown);
        walking_bindings.insert(InputAction::TurnLeft, KeyCode::ArrowLeft);
        walking_bindings.insert(InputAction::TurnRight, KeyCode::ArrowRight);
        walking_bindings.insert(InputAction::Run, KeyCode::ShiftLeft);
        walking_bindings.insert(InputAction::Interact, KeyCode::KeyF);
        walking_bindings.insert(InputAction::DebugInfo, KeyCode::F1);
        walking_bindings.insert(InputAction::EmergencyReset, KeyCode::F2);
        config.bindings.insert(GameState::Walking, walking_bindings);
        
        // Driving state bindings
        let mut driving_bindings = HashMap::new();
        driving_bindings.insert(InputAction::Forward, KeyCode::ArrowUp);
        driving_bindings.insert(InputAction::Backward, KeyCode::ArrowDown);
        driving_bindings.insert(InputAction::TurnLeft, KeyCode::ArrowLeft);
        driving_bindings.insert(InputAction::TurnRight, KeyCode::ArrowRight);
        driving_bindings.insert(InputAction::Turbo, KeyCode::Space);
        driving_bindings.insert(InputAction::Interact, KeyCode::KeyF);
        driving_bindings.insert(InputAction::DebugInfo, KeyCode::F1);
        driving_bindings.insert(InputAction::EmergencyReset, KeyCode::F2);
        config.bindings.insert(GameState::Driving, driving_bindings);
        
        // Flying state bindings (helicopter)
        let mut flying_bindings = HashMap::new();
        flying_bindings.insert(InputAction::Forward, KeyCode::ArrowUp);
        flying_bindings.insert(InputAction::Backward, KeyCode::ArrowDown);
        flying_bindings.insert(InputAction::TurnLeft, KeyCode::ArrowLeft);
        flying_bindings.insert(InputAction::TurnRight, KeyCode::ArrowRight);
        flying_bindings.insert(InputAction::VerticalUp, KeyCode::ShiftLeft);
        flying_bindings.insert(InputAction::VerticalDown, KeyCode::ControlLeft);
        flying_bindings.insert(InputAction::Interact, KeyCode::KeyF);
        flying_bindings.insert(InputAction::DebugInfo, KeyCode::F1);
        flying_bindings.insert(InputAction::EmergencyReset, KeyCode::F2);
        config.bindings.insert(GameState::Flying, flying_bindings);
        
        // Jetting state bindings (F16)
        let mut jetting_bindings = HashMap::new();
        jetting_bindings.insert(InputAction::PitchUp, KeyCode::KeyW);
        jetting_bindings.insert(InputAction::PitchDown, KeyCode::KeyS);
        jetting_bindings.insert(InputAction::RollLeft, KeyCode::KeyA);
        jetting_bindings.insert(InputAction::RollRight, KeyCode::KeyD);
        jetting_bindings.insert(InputAction::YawLeft, KeyCode::KeyQ);
        jetting_bindings.insert(InputAction::YawRight, KeyCode::KeyE);
        jetting_bindings.insert(InputAction::Forward, KeyCode::ArrowUp);
        jetting_bindings.insert(InputAction::Backward, KeyCode::ArrowDown);
        jetting_bindings.insert(InputAction::TurnLeft, KeyCode::ArrowLeft);
        jetting_bindings.insert(InputAction::TurnRight, KeyCode::ArrowRight);
        jetting_bindings.insert(InputAction::VerticalUp, KeyCode::ShiftLeft);
        jetting_bindings.insert(InputAction::VerticalDown, KeyCode::ControlLeft);
        jetting_bindings.insert(InputAction::Afterburner, KeyCode::Space);
        jetting_bindings.insert(InputAction::Interact, KeyCode::KeyF);
        jetting_bindings.insert(InputAction::DebugInfo, KeyCode::F1);
        jetting_bindings.insert(InputAction::EmergencyReset, KeyCode::F2);
        config.bindings.insert(GameState::Jetting, jetting_bindings);
        
        config
    }
}

impl InputConfig {
    pub fn get_key_for_action(&self, state: &GameState, action: InputAction) -> Option<KeyCode> {
        self.bindings
            .get(state)
            .and_then(|state_bindings| state_bindings.get(&action))
            .copied()
    }
    
    pub fn set_key_for_action(&mut self, state: GameState, action: InputAction, key: KeyCode) -> Result<(), String> {
        // Validate no conflicts within the same state
        if let Some(state_bindings) = self.bindings.get(&state) {
            for (existing_action, existing_key) in state_bindings.iter() {
                if *existing_key == key && *existing_action != action {
                    return Err(format!("Key {:?} already bound to {:?} in state {:?}", key, existing_action, state));
                }
            }
        }
        
        self.bindings
            .entry(state)
            .or_insert_with(HashMap::new)
            .insert(action, key);
        
        Ok(())
    }
    
    pub fn reset_to_defaults(&mut self) {
        *self = InputConfig::default();
    }
    
    pub fn enable_fallback(&mut self) {
        self.use_fallback = true;
    }
    
    pub fn disable_fallback(&mut self) {
        self.use_fallback = false;
    }
    
    pub fn is_fallback_enabled(&self) -> bool {
        self.use_fallback
    }
    
    /// Get all bindings for a specific state
    pub fn get_state_bindings(&self, state: &GameState) -> Option<&HashMap<InputAction, KeyCode>> {
        self.bindings.get(state)
    }
    
    /// Check if an action is available in a given state
    pub fn is_action_available(&self, state: &GameState, action: InputAction) -> bool {
        self.bindings
            .get(state)
            .map(|bindings| bindings.contains_key(&action))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = InputConfig::default();
        assert_eq!(config.get_key_for_action(&GameState::Walking, InputAction::Forward), Some(KeyCode::ArrowUp));
        assert_eq!(config.get_key_for_action(&GameState::Driving, InputAction::Turbo), Some(KeyCode::Space));
    }
    
    #[test]
    fn test_conflict_detection() {
        let mut config = InputConfig::default();
        let result = config.set_key_for_action(GameState::Walking, InputAction::Backward, KeyCode::ArrowUp);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_valid_binding_change() {
        let mut config = InputConfig::default();
        let result = config.set_key_for_action(GameState::Walking, InputAction::Forward, KeyCode::KeyW);
        assert!(result.is_ok());
        assert_eq!(config.get_key_for_action(&GameState::Walking, InputAction::Forward), Some(KeyCode::KeyW));
    }
}
