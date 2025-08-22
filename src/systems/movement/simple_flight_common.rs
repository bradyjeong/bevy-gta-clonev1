use crate::components::ControlState;
use bevy::prelude::*;

/// Shared flight physics utilities to eliminate boilerplate between helicopter and F-16
pub struct SimpleFlightCommon;

impl SimpleFlightCommon {
    /// Get stable timestep - prevents physics instability from frame rate spikes
    pub fn stable_dt(time: &Res<Time>) -> f32 {
        time.delta_secs().min(0.05) // Cap at 20 FPS minimum
    }

    /// Process throttle input with brake-to-zero logic
    pub fn process_throttle(
        control_state: &ControlState,
        current_throttle: f32,
        throttle_up_rate: f32,
        throttle_down_rate: f32,
        dt: f32,
    ) -> f32 {
        // Brake forces throttle to zero
        if control_state.brake > 0.1 {
            return 0.0;
        }

        // Normal throttle control
        let target_throttle = control_state.throttle;
        let rate = if target_throttle > current_throttle {
            throttle_up_rate
        } else {
            throttle_down_rate
        };

        current_throttle + (target_throttle - current_throttle) * rate * dt
    }

    /// Calculate flame intensity from flight state
    pub fn calculate_flame_intensity(throttle: f32, afterburner_active: bool) -> f32 {
        let base_intensity = throttle;
        let afterburner_boost = if afterburner_active { 0.8 } else { 0.0 };
        (base_intensity + afterburner_boost).clamp(0.0, 1.0)
    }
}
