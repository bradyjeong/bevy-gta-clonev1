//! ControlManager Extension Trait - Phase 3 API Convenience Layer
//! 
//! Provides convenience methods for the ActionState<GameAction> to bridge
//! the gap between leafwing-input-manager and our game-specific control logic.

use leafwing_input_manager::prelude::*;
use crate::input::control_manager::GameAction;

pub trait ControlManagerExt {
    /// Check if a control action is currently active (pressed or just pressed)
    fn is_control_active(&self, action: GameAction) -> bool;
    
    /// Get the current axis value for analog inputs
    fn axis(&self, axis: GameAction) -> f32;
    
    /// Convenience method for checking if accelerating
    fn is_accelerating(&self) -> bool;
    
    /// Convenience method for checking if braking
    fn is_braking(&self) -> bool;
    
    /// Convenience method for checking if steering left
    fn is_steering_left(&self) -> bool;
    
    /// Convenience method for checking if steering right
    fn is_steering_right(&self) -> bool;
}

impl ControlManagerExt for ActionState<GameAction> {
    fn is_control_active(&self, action: GameAction) -> bool {
        self.just_pressed(action) || self.pressed(action)
    }
    
    fn axis(&self, axis: GameAction) -> f32 {
        self.value(axis)
    }
    
    fn is_accelerating(&self) -> bool {
        self.is_control_active(GameAction::Accelerate)
    }
    
    fn is_braking(&self) -> bool {
        self.is_control_active(GameAction::Brake)
    }
    
    fn is_steering_left(&self) -> bool {
        self.is_control_active(GameAction::SteerLeft)
    }
    
    fn is_steering_right(&self) -> bool {
        self.is_control_active(GameAction::SteerRight)
    }
}
