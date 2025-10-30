use bevy::prelude::*;

/// Simple movement tracking component for entities
/// Tracks entity position changes for potential future use
#[derive(Component, Clone)]
pub struct MovementTracker {
    pub last_position: Vec3,
    pub movement_threshold: f32,
}

impl Default for MovementTracker {
    fn default() -> Self {
        Self {
            last_position: Vec3::ZERO,
            movement_threshold: 1.0,
        }
    }
}

impl MovementTracker {
    pub fn new(position: Vec3, threshold: f32) -> Self {
        Self {
            last_position: position,
            movement_threshold: threshold,
        }
    }

    pub fn has_moved_significantly(&self, current_position: Vec3) -> bool {
        self.last_position.distance(current_position) > self.movement_threshold
    }

    pub fn update_position(&mut self, position: Vec3) {
        self.last_position = position;
    }
}
