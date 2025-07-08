//! ───────────────────────────────────────────────
//! System:   `ControlManager` Extensions
//! Purpose:  Extension methods for `ControlManager`
//! Schedule: Update
//! Reads:    Input states
//! Writes:   Control states
//! Invariants:
//!   * Control states remain consistent
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use crate::input::ControlManager;

/// Extension trait for `ControlManager` providing additional input methods
pub trait ControlManagerExt {
    fn is_accelerating(&self) -> bool;
    fn is_braking(&self) -> bool;
    fn is_turning_left(&self) -> bool;
    fn is_turning_right(&self) -> bool;
    fn is_moving_forward(&self) -> bool;
    fn is_moving_backward(&self) -> bool;
    fn axis(&self, axis_name: &str) -> f32;
    fn get_steering_input(&self) -> f32;
    fn get_throttle_input(&self) -> f32;
    fn get_brake_input(&self) -> f32;
    fn get_pitch_input(&self) -> f32;
    fn get_yaw_input(&self) -> f32;
    fn get_roll_input(&self) -> f32;
}

impl ControlManagerExt for ControlManager {
    fn is_accelerating(&self) -> bool {
        todo!("Phase 4: Implement is_accelerating")
    }

    fn is_braking(&self) -> bool {
        todo!("Phase 4: Implement is_braking")
    }

    fn is_turning_left(&self) -> bool {
        todo!("Phase 4: Implement is_turning_left")
    }

    fn is_turning_right(&self) -> bool {
        todo!("Phase 4: Implement is_turning_right")
    }

    fn is_moving_forward(&self) -> bool {
        todo!("Phase 4: Implement is_moving_forward")
    }

    fn is_moving_backward(&self) -> bool {
        todo!("Phase 4: Implement is_moving_backward")
    }

    fn axis(&self, _axis_name: &str) -> f32 {
        todo!("Phase 4: Implement axis")
    }

    fn get_steering_input(&self) -> f32 {
        todo!("Phase 4: Implement get_steering_input")
    }

    fn get_throttle_input(&self) -> f32 {
        todo!("Phase 4: Implement get_throttle_input")
    }

    fn get_brake_input(&self) -> f32 {
        todo!("Phase 4: Implement get_brake_input")
    }

    fn get_pitch_input(&self) -> f32 {
        todo!("Phase 4: Implement get_pitch_input")
    }

    fn get_yaw_input(&self) -> f32 {
        todo!("Phase 4: Implement get_yaw_input")
    }

    fn get_roll_input(&self) -> f32 {
        todo!("Phase 4: Implement get_roll_input")
    }
}

// Export functions for backward compatibility
#[must_use] pub fn is_accelerating(control_manager: &ControlManager) -> bool {
    control_manager.is_accelerating()
}

#[must_use] pub fn is_braking(control_manager: &ControlManager) -> bool {
    control_manager.is_braking()
}

#[must_use] pub fn is_turning_left(control_manager: &ControlManager) -> bool {
    control_manager.is_turning_left()
}

#[must_use] pub fn is_turning_right(control_manager: &ControlManager) -> bool {
    control_manager.is_turning_right()
}
