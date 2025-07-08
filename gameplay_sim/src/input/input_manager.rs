//! ───────────────────────────────────────────────
//! System:   Input Manager
//! Purpose:  Processes user input and control mapping
//! Schedule: Update
//! Reads:    InputCompatLayer, InputConfig, InputManager
//! Writes:   InputCompatLayer, InputManager
//! Invariants:
//!   * Only active entities can be controlled
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use std::collections::HashSet;
use std::time::Instant;
use game_core::game_state::GameState;
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
            for (action, key) in state_bindings.iter() {
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
        
        // Fallback mode: if config fails, use hardcoded bindings
        if config.is_fallback_enabled() || new_active_actions.is_empty() {
            self.process_fallback_input(input, current_state, &mut new_active_actions);
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
    
    /// Fallback input processing using hardcoded bindings
    fn process_fallback_input(
        &mut self,
        input: &ButtonInput<KeyCode>,
        current_state: &GameState,
        active_actions: &mut HashSet<InputAction>,
    ) {
        match current_state {
            GameState::Walking => {
                if input.pressed(KeyCode::ArrowUp) { active_actions.insert(InputAction::Forward); }
                if input.pressed(KeyCode::ArrowDown) { active_actions.insert(InputAction::Backward); }
                if input.pressed(KeyCode::ArrowLeft) { active_actions.insert(InputAction::TurnLeft); }
                if input.pressed(KeyCode::ArrowRight) { active_actions.insert(InputAction::TurnRight); }
                if input.pressed(KeyCode::ShiftLeft) { active_actions.insert(InputAction::Run); }
                if input.just_pressed(KeyCode::KeyF) { 
                    active_actions.insert(InputAction::Interact);
                    self.just_pressed_actions.insert(InputAction::Interact);
                }
            }
            GameState::Driving => {
                if input.pressed(KeyCode::ArrowUp) { active_actions.insert(InputAction::Forward); }
                if input.pressed(KeyCode::ArrowDown) { active_actions.insert(InputAction::Backward); }
                if input.pressed(KeyCode::ArrowLeft) { active_actions.insert(InputAction::TurnLeft); }
                if input.pressed(KeyCode::ArrowRight) { active_actions.insert(InputAction::TurnRight); }
                if input.pressed(KeyCode::Space) { active_actions.insert(InputAction::Turbo); }
                if input.just_pressed(KeyCode::KeyF) { 
                    active_actions.insert(InputAction::Interact);
                    self.just_pressed_actions.insert(InputAction::Interact);
                }
            }
            GameState::Flying => {
                if input.pressed(KeyCode::ArrowUp) { active_actions.insert(InputAction::Forward); }
                if input.pressed(KeyCode::ArrowDown) { active_actions.insert(InputAction::Backward); }
                if input.pressed(KeyCode::ArrowLeft) { active_actions.insert(InputAction::TurnLeft); }
                if input.pressed(KeyCode::ArrowRight) { active_actions.insert(InputAction::TurnRight); }
                if input.pressed(KeyCode::ShiftLeft) { active_actions.insert(InputAction::VerticalUp); }
                if input.pressed(KeyCode::ControlLeft) { active_actions.insert(InputAction::VerticalDown); }
                if input.just_pressed(KeyCode::KeyF) { 
                    active_actions.insert(InputAction::Interact);
                    self.just_pressed_actions.insert(InputAction::Interact);
                }
            }
            GameState::Jetting => {
                if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) { active_actions.insert(InputAction::PitchUp); }
                if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) { active_actions.insert(InputAction::PitchDown); }
                if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) { active_actions.insert(InputAction::RollLeft); }
                if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) { active_actions.insert(InputAction::RollRight); }
                if input.pressed(KeyCode::KeyQ) { active_actions.insert(InputAction::YawLeft); }
                if input.pressed(KeyCode::KeyE) { active_actions.insert(InputAction::YawRight); }
                if input.pressed(KeyCode::ShiftLeft) { active_actions.insert(InputAction::VerticalUp); }
                if input.pressed(KeyCode::ControlLeft) { active_actions.insert(InputAction::VerticalDown); }
                if input.pressed(KeyCode::Space) { active_actions.insert(InputAction::Afterburner); }
                if input.just_pressed(KeyCode::KeyF) { 
                    active_actions.insert(InputAction::Interact);
                    self.just_pressed_actions.insert(InputAction::Interact);
                }
            }
        }
        
        // Common debug actions
        if input.just_pressed(KeyCode::F1) { 
            active_actions.insert(InputAction::DebugInfo);
            self.just_pressed_actions.insert(InputAction::DebugInfo);
        }
        if input.just_pressed(KeyCode::F2) { 
            active_actions.insert(InputAction::EmergencyReset);
            self.just_pressed_actions.insert(InputAction::EmergencyReset);
        }
    }
    
    /// Check if an action is currently pressed
    pub fn is_action_pressed(&self, action: InputAction) -> bool {
        self.active_actions.contains(&action)
    }
    
    /// Check if an action was just pressed this frame
    pub fn is_action_just_pressed(&self, action: InputAction) -> bool {
        self.just_pressed_actions.contains(&action)
    }
    
    /// Check if an action was just released this frame
    pub fn is_action_just_released(&self, action: InputAction) -> bool {
        self.just_released_actions.contains(&action)
    }
    
    /// Get all currently active actions
    pub fn get_active_actions(&self) -> &HashSet<InputAction> {
        &self.active_actions
    }
    
    /// Get performance statistics
    pub fn get_performance_stats(&self) -> (u128, u64) {
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
    input_manager.process_input(&input, &input_config, &**current_state);
}

/// Backwards compatibility layer - provides the same interface as raw KeyCode input
#[derive(Resource)]
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

impl Default for InputCompatLayer {
    fn default() -> Self {
        Self {
            arrow_up: false,
            arrow_down: false,
            arrow_left: false,
            arrow_right: false,
            shift_left: false,
            control_left: false,
            space: false,
            key_f: false,
            key_w: false,
            key_s: false,
            key_a: false,
            key_d: false,
            key_q: false,
            key_e: false,
            f1: false,
            f2: false,
            f_just_pressed: false,
            f1_just_pressed: false,
            f2_just_pressed: false,
        }
    }
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
            compat_layer.arrow_up = input_manager.is_action_pressed(InputAction::Forward);
            compat_layer.arrow_down = input_manager.is_action_pressed(InputAction::Backward);
            compat_layer.arrow_left = input_manager.is_action_pressed(InputAction::TurnLeft);
            compat_layer.arrow_right = input_manager.is_action_pressed(InputAction::TurnRight);
            compat_layer.space = input_manager.is_action_pressed(InputAction::Turbo);
        }
        GameState::Flying => {
            compat_layer.arrow_up = input_manager.is_action_pressed(InputAction::Forward);
            compat_layer.arrow_down = input_manager.is_action_pressed(InputAction::Backward);
            compat_layer.arrow_left = input_manager.is_action_pressed(InputAction::TurnLeft);
            compat_layer.arrow_right = input_manager.is_action_pressed(InputAction::TurnRight);
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
            compat_layer.shift_left = input_manager.is_action_pressed(InputAction::VerticalUp);
            compat_layer.control_left = input_manager.is_action_pressed(InputAction::VerticalDown);
            compat_layer.space = input_manager.is_action_pressed(InputAction::Afterburner);
        }
    }
    
    // Common actions
    compat_layer.key_f = input_manager.is_action_pressed(InputAction::Interact);
    compat_layer.f1 = input_manager.is_action_pressed(InputAction::DebugInfo);
    compat_layer.f2 = input_manager.is_action_pressed(InputAction::EmergencyReset);
    
    // Just pressed states
    compat_layer.f_just_pressed = input_manager.is_action_just_pressed(InputAction::Interact);
    compat_layer.f1_just_pressed = input_manager.is_action_just_pressed(InputAction::DebugInfo);
    compat_layer.f2_just_pressed = input_manager.is_action_just_pressed(InputAction::EmergencyReset);
}
