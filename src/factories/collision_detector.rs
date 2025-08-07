use bevy::prelude::*;
use crate::components::ContentType;

/// Collision detector for entity spawning
/// 
/// Follows AGENT.md principles:
/// - Single responsibility: Detects collisions between entities during spawning
/// - Clear boundaries: Only handles collision detection, no spawning logic
/// - Minimal coupling: Only depends on basic position and type data
/// - Straightforward algorithm: Distance-based collision detection
pub struct CollisionDetector;

impl CollisionDetector {
    /// Check for content collision with existing entities
    /// 
    /// Simple collision detection following AGENT.md:
    /// - Explicit distance rules: Clear minimum distances per content type
    /// - No complex algorithms: Basic distance checking
    /// - Configurable buffer: 2.0m safety buffer for all entities
    pub fn has_collision(
        position: Vec3, 
        content_type: ContentType,
        existing_content: &[(Vec3, ContentType, f32)]
    ) -> bool {
        let min_distance = match content_type {
            ContentType::Building => 35.0, // Buildings need significant space
            ContentType::Vehicle => 25.0,  // Vehicles need road clearance
            ContentType::Tree => 10.0,     // Trees can be closer together
            ContentType::NPC => 5.0,       // NPCs are small and mobile
            _ => 15.0,                     // Default safety distance
        };
        
        existing_content.iter().any(|(existing_pos, _, radius)| {
            let required_distance = min_distance + radius + 2.0; // 2.0m safety buffer
            position.distance(*existing_pos) < required_distance
        })
    }
    
    /// Check collision between two specific entities
    /// 
    /// Helper method for more precise collision checking:
    /// - Considers both entity types and sizes
    /// - Returns collision distance for debugging
    pub fn check_entity_collision(
        pos1: Vec3,
        type1: ContentType, 
        radius1: f32,
        pos2: Vec3,
        type2: ContentType,
        radius2: f32
    ) -> Option<f32> {
        let min_distance_1 = Self::get_min_distance(type1);
        let min_distance_2 = Self::get_min_distance(type2);
        let required_distance = (min_distance_1 + min_distance_2) / 2.0 + radius1 + radius2 + 2.0;
        
        let actual_distance = pos1.distance(pos2);
        if actual_distance < required_distance {
            Some(required_distance - actual_distance) // How much they overlap
        } else {
            None // No collision
        }
    }
    
    /// Get minimum distance for content type
    /// 
    /// Private helper that centralizes distance rules:
    /// - Consistent with has_collision method
    /// - Easy to tune for gameplay balance
    fn get_min_distance(content_type: ContentType) -> f32 {
        match content_type {
            ContentType::Building => 35.0,
            ContentType::Vehicle => 25.0,
            ContentType::Tree => 10.0,
            ContentType::NPC => 5.0,
            _ => 15.0,
        }
    }
}
