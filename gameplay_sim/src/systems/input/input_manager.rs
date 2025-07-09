//! ───────────────────────────────────────────────
//! System:   Input Manager
//! Purpose:  Processes user input and control mapping
//! Schedule: Update
//! Reads:    `InputConfig`, `InputManager`
//! Writes:   `InputManager`
//! Invariants:
//!   * Only active entities can be controlled
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use std::collections::HashSet;
use std::time::Instant;
use game_core::prelude::*;
use super::input_config::{InputConfig, InputAction};

#[derive(Resource, Default)]
pub struct InputManager {
    // Track active actions this frame
    active_actions: HashSet<InputAction>,
    just_pressed_actions: HashSet<InputAction>,
    just_released_actions: HashSet<InputAction>,
    
    // Performance monitoring
    last_process_time: Option<Instant>,
    max_process_time_us: u128,
    frame_count: u64,
}
impl InputManager {
    /// Process input for the current frame - PERFORMANCE CRITICAL <1ms
    pub fn process_input(
        &mut self,
        input: &ButtonInput<KeyCode>,
        config: &InputConfig,
        current_state: &GameState,
    ) {
        let start_time = Instant::now();
        
        // Clear previous frame data
        self.just_pressed_actions.clear();
        self.just_released_actions.clear();
        let mut new_active_actions = HashSet::new();
        
        // Get bindings for current state
        if let Some(state_bindings) = config.get_state_bindings(current_state) {
            // Fast path: iterate through configured bindings only
            for (action, key) in state_bindings {
                let was_active = self.active_actions.contains(action);
                let is_active = input.pressed(*key);
                
                if is_active {
                    new_active_actions.insert(*action);
                    
                    if !was_active {
                        self.just_pressed_actions.insert(*action);
                    }
                } else if was_active {
                    self.just_released_actions.insert(*action);
                }
            }
        }
        

        
        self.active_actions = new_active_actions;
        
        // Performance monitoring
        let process_time = start_time.elapsed().as_micros();
        self.max_process_time_us = self.max_process_time_us.max(process_time);
        self.frame_count += 1;
        self.last_process_time = Some(start_time);
        
        // Warn if processing takes too long
        if process_time > 1000 { // 1ms = 1000 microseconds
            warn!("Input processing took {}μs (>1ms limit)", process_time);
        }
    }

    /// Check if an action is currently pressed
    #[must_use] pub fn is_action_pressed(&self, action: InputAction) -> bool {
        self.active_actions.contains(&action)
    }
    
    /// Check if an action was just pressed this frame
    #[must_use] pub fn is_action_just_pressed(&self, action: InputAction) -> bool {
        self.just_pressed_actions.contains(&action)
    }
    
    /// Check if an action was just released this frame
    #[must_use] pub fn is_action_just_released(&self, action: InputAction) -> bool {
        self.just_released_actions.contains(&action)
    }
    
    /// Get all currently active actions
    #[must_use] pub fn get_active_actions(&self) -> &HashSet<InputAction> {
        &self.active_actions
    }
    
    /// Get performance statistics
    #[must_use] pub fn get_performance_stats(&self) -> (u128, u64) {
        (self.max_process_time_us, self.frame_count)
    }
    
    /// Reset performance statistics
    pub fn reset_performance_stats(&mut self) {
        self.max_process_time_us = 0;
        self.frame_count = 0;
    }
    
    /// Clear all input state (useful for emergency reset)
    pub fn clear_all_input(&mut self) {
        self.active_actions.clear();
        self.just_pressed_actions.clear();
        self.just_released_actions.clear();
    }
}
/// System to process input each frame
pub fn process_input_system(
    input: Res<ButtonInput<KeyCode>>,
    mut input_manager: ResMut<InputManager>,
    input_config: Res<InputConfig>,
    current_state: Res<State<GameState>>,
) {
    input_manager.process_input(&input, &input_config, &current_state);
}

/// Backwards compatibility layer - provides the same interface as raw `KeyCode` input
#[derive(Resource)]
#[derive(Default)]
pub struct InputCompatLayer {
    // Cache the last input state for backwards compatibility
    pub arrow_up: bool,
    pub arrow_down: bool,
    pub arrow_left: bool,
    pub arrow_right: bool,
    pub shift_left: bool,
    pub control_left: bool,
    pub space: bool,
    pub key_f: bool,
    pub key_w: bool,
    pub key_s: bool,
    pub key_a: bool,
    pub key_d: bool,
    pub key_q: bool,
    pub key_e: bool,
    pub f1: bool,
    pub f2: bool,
    // Just pressed states
    pub f_just_pressed: bool,
    pub f1_just_pressed: bool,
    pub f2_just_pressed: bool,
}

/// System to update backwards compatibility layer
pub fn update_input_compat_layer(
    input_manager: Res<InputManager>,
    mut compat_layer: ResMut<InputCompatLayer>,
    current_state: Res<State<GameState>>,
) {
    // Map actions back to key states for backwards compatibility
    match **current_state {
        GameState::Walking => {
            compat_layer.arrow_up = input_manager.is_action_pressed(InputAction::Forward);
            compat_layer.arrow_down = input_manager.is_action_pressed(InputAction::Backward);
            compat_layer.arrow_left = input_manager.is_action_pressed(InputAction::TurnLeft);
            compat_layer.arrow_right = input_manager.is_action_pressed(InputAction::TurnRight);
            compat_layer.shift_left = input_manager.is_action_pressed(InputAction::Run);
        }
        GameState::Driving => {
            compat_layer.space = input_manager.is_action_pressed(InputAction::Turbo);
        }
        GameState::Flying => {
            compat_layer.shift_left = input_manager.is_action_pressed(InputAction::VerticalUp);
            compat_layer.control_left = input_manager.is_action_pressed(InputAction::VerticalDown);
        }
        GameState::Jetting => {
            compat_layer.key_w = input_manager.is_action_pressed(InputAction::PitchUp);
            compat_layer.key_s = input_manager.is_action_pressed(InputAction::PitchDown);
            compat_layer.key_a = input_manager.is_action_pressed(InputAction::RollLeft);
            compat_layer.key_d = input_manager.is_action_pressed(InputAction::RollRight);
            compat_layer.key_q = input_manager.is_action_pressed(InputAction::YawLeft);
            compat_layer.key_e = input_manager.is_action_pressed(InputAction::YawRight);
            compat_layer.arrow_up = input_manager.is_action_pressed(InputAction::PitchUp);
            compat_layer.arrow_down = input_manager.is_action_pressed(InputAction::PitchDown);
            compat_layer.arrow_left = input_manager.is_action_pressed(InputAction::RollLeft);
            compat_layer.arrow_right = input_manager.is_action_pressed(InputAction::RollRight);
            compat_layer.space = input_manager.is_action_pressed(InputAction::Afterburner);
        }
    }
    
    // Common actions
    compat_layer.key_f = input_manager.is_action_pressed(InputAction::Interact);
    compat_layer.f1 = input_manager.is_action_pressed(InputAction::DebugInfo);
    compat_layer.f2 = input_manager.is_action_pressed(InputAction::EmergencyReset);
    compat_layer.f_just_pressed = input_manager.is_action_just_pressed(InputAction::Interact);
    compat_layer.f1_just_pressed = input_manager.is_action_just_pressed(InputAction::DebugInfo);
    compat_layer.f2_just_pressed = input_manager.is_action_just_pressed(InputAction::EmergencyReset);
}
