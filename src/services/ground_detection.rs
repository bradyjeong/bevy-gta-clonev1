use bevy::prelude::*;

/// Service for ground height detection
/// Uses simple terrain estimation for spawning entities at correct height
#[derive(Resource)]
pub struct GroundDetectionService {
    pub enabled: bool,
    pub fallback_height: f32,
}

impl Default for GroundDetectionService {
    fn default() -> Self {
        Self {
            enabled: true,
            fallback_height: 0.05, // Match terrain collider top surface
        }
    }
}

impl GroundDetectionService {
    /// Get ground height without physics raycasting
    /// Uses simple terrain estimation matching the actual terrain from setup_basic_world
    /// Terrain is at y=0.0, collider top surface is at 0.05
    pub fn get_ground_height_simple(&self, _position: Vec2) -> f32 {
        if !self.enabled {
            return self.fallback_height;
        }
        0.05 // Flat ground at collider top surface
    }

    /// Check if a position is suitable for NPC spawning
    /// Avoids spawning too close to origin where roads are
    pub fn is_spawn_position_valid(&self, position: Vec2) -> bool {
        if !self.enabled {
            return true;
        }

        // Simple validation - avoid positions too close to origin (where roads typically are)
        let distance_from_origin = position.length();
        distance_from_origin > 10.0 // Stay away from central road network
    }
}

/// Plugin to add ground detection service
pub struct GroundDetectionPlugin;

impl Plugin for GroundDetectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GroundDetectionService>();
    }
}
