use bevy::prelude::*;

/// Shared movement tracking component to break circular dependencies
#[derive(Component, Default, Debug, Clone)]
pub struct SharedMovementTracker {
    pub last_position: Vec3,
    pub velocity: Vec3,
    pub distance_traveled: f32,
    pub last_update_time: f32,
}

impl SharedMovementTracker {
    pub fn new(position: Vec3) -> Self {
        Self {
            last_position: position,
            velocity: Vec3::ZERO,
            distance_traveled: 0.0,
            last_update_time: 0.0,
        }
    }
    
    pub fn update(&mut self, new_position: Vec3, current_time: f32) {
        let delta_time = current_time - self.last_update_time;
        if delta_time > 0.0 {
            let distance = self.last_position.distance(new_position);
            self.distance_traveled += distance;
            self.velocity = (new_position - self.last_position) / delta_time;
            self.last_position = new_position;
            self.last_update_time = current_time;
        }
    }
}
