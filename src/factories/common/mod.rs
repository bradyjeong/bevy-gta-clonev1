//! Common utilities shared across all focused factories
//! 
//! Following AGENT.md simplicity principles:
//! - Single responsibility per module
//! - Stateless utility functions
//! - Clear, minimal interfaces

use bevy::prelude::*;

pub mod spawn_utils;
pub mod physics_setup;

pub use spawn_utils::*;
pub use physics_setup::*;

/// Common trait for all focused factories
pub trait FocusedFactory {
    /// Get the factory name for debugging
    fn name() -> &'static str;
    
    /// Get entity limits for this factory type
    fn entity_limit() -> usize;
}

/// Common validation utilities
pub struct ValidationUtils;

impl ValidationUtils {
    /// Validate position is within world bounds
    pub fn validate_position(position: Vec3, max_coord: f32) -> Result<Vec3, String> {
        if position.x.abs() > max_coord || position.z.abs() > max_coord {
            return Err(format!(
                "Position {:?} exceeds world bounds ±{}", 
                position, max_coord
            ));
        }
        
        Ok(position.clamp(
            Vec3::splat(-max_coord),
            Vec3::splat(max_coord),
        ))
    }
    
    /// Check if position has collision with existing entities
    pub fn has_collision(
        position: Vec3,
        min_distance: f32,
        existing_content: &[(Vec3, f32)]
    ) -> bool {
        existing_content.iter().any(|(existing_pos, radius)| {
            let required_distance = min_distance + radius + 2.0; // 2.0 buffer
            position.distance(*existing_pos) < required_distance
        })
    }
}
