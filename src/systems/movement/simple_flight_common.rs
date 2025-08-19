use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::config::GameConfig;
use crate::components::ControlState;

/// Shared flight physics utilities to eliminate boilerplate between helicopter and F-16
pub struct SimpleFlightCommon;

impl SimpleFlightCommon {
    /// Get stable timestep - prevents physics instability from frame rate spikes
    pub fn stable_dt(time: &Res<Time>) -> f32 {
        time.delta_secs().min(0.05) // Cap at 20 FPS minimum
    }
    
    /// Apply safety clamps only - let Rapier handle gravity to avoid duplication
    pub fn apply_velocity_clamps(
        velocity: &mut Velocity,
        config: &GameConfig,
    ) {
        // Safety clamps to prevent physics solver issues
        let max_vel = config.physics.max_velocity;
        velocity.linvel = velocity.linvel.clamp(
            Vec3::splat(-max_vel),
            Vec3::splat(max_vel)
        );
        
        let max_ang_vel = config.physics.max_angular_velocity;
        velocity.angvel = velocity.angvel.clamp(
            Vec3::splat(-max_ang_vel),
            Vec3::splat(max_ang_vel)
        );
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
    pub fn calculate_flame_intensity(
        throttle: f32,
        afterburner_active: bool,
    ) -> f32 {
        let base_intensity = throttle;
        let afterburner_boost = if afterburner_active { 0.8 } else { 0.0 };
        (base_intensity + afterburner_boost).clamp(0.0, 1.0)
    }
}
