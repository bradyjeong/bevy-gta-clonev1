//! Bevy adapters for engine abstractions

use bevy::prelude::*;

/// Trait for physics world abstraction
pub trait PhysicsWorld {
    /// Validate position within world bounds
    fn validate_position(&self, position: Vec3) -> Vec3;
    /// Validate velocity within limits
    fn validate_velocity(&self, velocity: Vec3) -> Vec3;
}

/// Trait for audio system abstraction
pub trait AudioSink {
    /// Play a sound effect
    fn play_sound(&mut self, sound_id: &str);
}

/// Trait for save/load system abstraction
pub trait SaveStorage {
    /// Save data to storage
    fn save(&mut self, key: &str, data: &[u8]) -> Result<(), String>;
    /// Load data from storage
    fn load(&self, key: &str) -> Result<Vec<u8>, String>;
}
