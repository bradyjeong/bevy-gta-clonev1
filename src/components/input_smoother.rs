use bevy::prelude::*;

/// Smooths raw input to prevent jolting from instant velocity changes
///
/// Uses exponential moving average to create natural-feeling acceleration/deceleration
/// Similar to GTA V's input buffering system
#[derive(Component, Debug, Clone)]
pub struct InputSmoother {
    pub smoothed_throttle: f32,
    pub smoothed_brake: f32,
    pub smoothed_reverse: f32,
    pub smoothed_steering: f32,
    pub smoothed_vertical: f32,
    pub smoothed_yaw: f32,
    pub smoothed_roll: f32,
    pub smoothed_pitch: f32,
    pub smoothed_boost: f32,

    pub smoothing_speed: f32,
}

impl Default for InputSmoother {
    fn default() -> Self {
        Self {
            smoothed_throttle: 0.0,
            smoothed_brake: 0.0,
            smoothed_reverse: 0.0,
            smoothed_steering: 0.0,
            smoothed_vertical: 0.0,
            smoothed_yaw: 0.0,
            smoothed_roll: 0.0,
            smoothed_pitch: 0.0,
            smoothed_boost: 0.0,
            smoothing_speed: 8.0,
        }
    }
}

impl InputSmoother {
    pub fn new(smoothing_speed: f32) -> Self {
        Self {
            smoothing_speed,
            ..Default::default()
        }
    }

    pub fn smooth_value(&self, current: f32, target: f32, dt: f32) -> f32 {
        if !dt.is_finite() || dt <= 0.0 {
            return current;
        }
        let factor = (self.smoothing_speed * dt).clamp(0.0, 1.0);
        let result = current + (target - current) * factor;
        if result.is_finite() { result } else { current }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        raw_throttle: f32,
        raw_brake: f32,
        raw_reverse: f32,
        raw_steering: f32,
        raw_vertical: f32,
        raw_yaw: f32,
        raw_roll: f32,
        raw_pitch: f32,
        raw_boost: f32,
        dt: f32,
    ) {
        self.smoothed_throttle = self.smooth_value(self.smoothed_throttle, raw_throttle, dt);
        self.smoothed_brake = self.smooth_value(self.smoothed_brake, raw_brake, dt);
        self.smoothed_reverse = self.smooth_value(self.smoothed_reverse, raw_reverse, dt);
        self.smoothed_steering = self.smooth_value(self.smoothed_steering, raw_steering, dt);
        self.smoothed_vertical = self.smooth_value(self.smoothed_vertical, raw_vertical, dt);
        self.smoothed_yaw = self.smooth_value(self.smoothed_yaw, raw_yaw, dt);
        self.smoothed_roll = self.smooth_value(self.smoothed_roll, raw_roll, dt);
        self.smoothed_pitch = self.smooth_value(self.smoothed_pitch, raw_pitch, dt);
        self.smoothed_boost = self.smooth_value(self.smoothed_boost, raw_boost, dt);
    }
}
