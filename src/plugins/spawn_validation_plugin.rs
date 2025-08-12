//! Spawn Validation Plugin - P2 Architectural Shift
//! 
//! Consolidates all spawn validation logic into pure stateless functions.
//! Other plugins can call these directly (utility module pattern per AGENT.md §42 line 52).
//! 
//! Replaces scattered validation logic from:
//! - systems/spawn_validation.rs
//! - factories/position_validator.rs  
//! - factories/common/spawn_utils.rs
//! - factories/collision_detector.rs
//! - factories/entity_factory_unified.rs

use bevy::prelude::*;
use crate::components::ContentType;
use crate::world::RoadNetwork;

/// Pure stateless spawn validation functions
/// 
/// Following AGENT.md principles:
/// - Pure functions with no side effects
/// - Single responsibility per function
/// - Explicit over implicit validation rules
/// - No tangled dependencies
pub struct SpawnValidation;

impl SpawnValidation {
    /// Check if spawn position is valid for content type
    /// 
    /// Pure function that validates position based on:
    /// - World bounds
    /// - Road constraints (vehicles need roads, buildings avoid them)
    /// - Water area restrictions
    /// - Content-specific rules
    pub fn is_spawn_position_valid(
        position: Vec3,
        content_type: ContentType,
        max_world_coord: f32,
        road_network: Option<&RoadNetwork>,
    ) -> bool {
        // World bounds check
        if position.x.abs() > max_world_coord || position.z.abs() > max_world_coord {
            return false;
        }
        
        // Road constraint checks
        // NOTE: Direct RoadNetwork.is_near_road() is OK here as RoadNetwork is a shared resource.
        // For cross-plugin coordination, use RequestRoadValidation/RoadValidationResult events.
        if let Some(roads) = road_network {
            let road_tolerance = Self::get_road_tolerance(content_type);
            let on_road = roads.is_near_road(position, road_tolerance.abs());
            
            match content_type {
                ContentType::Vehicle => {
                    if !on_road { return false; } // Vehicles need roads
                }
                ContentType::Building | ContentType::Tree => {
                    if on_road { return false; } // Buildings/trees avoid roads
                }
                _ => {} // NPCs don't care about roads
            }
        }
        
        // Water area restrictions
        if Self::is_in_water_area(position) && !matches!(content_type, ContentType::Vehicle) {
            return false;
        }
        
        true
    }
    
    /// Check for collision with existing content
    /// 
    /// Pure function that detects entity overlaps using distance-based collision.
    /// Returns true if collision would occur.
    pub fn has_content_collision(
        position: Vec3,
        content_type: ContentType,
        existing_content: &[(Vec3, ContentType, f32)],
    ) -> bool {
        let min_distance = Self::get_collision_tolerance(content_type);
        
        existing_content.iter().any(|(existing_pos, _, radius)| {
            let required_distance = min_distance + radius + 2.0; // 2.0m safety buffer
            position.distance(*existing_pos) < required_distance
        })
    }
    
    /// Check if position is in water area
    /// 
    /// Pure function for water area detection.
    /// Lake parameters match water.rs setup.
    pub fn is_in_water_area(position: Vec3) -> bool {
        // Lake position and size (must match water.rs setup)
        let lake_center = Vec3::new(300.0, -2.0, 300.0);
        let lake_size = 200.0;
        let buffer = 20.0; // Extra buffer around lake
        
        let distance = Vec2::new(
            position.x - lake_center.x,
            position.z - lake_center.z,
        ).length();
        
        distance < (lake_size / 2.0 + buffer)
    }
    
    /// Check if position is on or near a road
    /// 
    /// Pure function that wraps road spline checking.
    /// Returns true if position is within tolerance of any road.
    /// 
    /// NOTE: Uses RoadNetwork directly as it's a shared resource.
    /// For cross-plugin event-based validation, use RequestRoadValidation/RoadValidationResult.
    pub fn is_on_road(
        position: Vec3,
        road_network: &RoadNetwork,
        tolerance: f32,
    ) -> bool {
        road_network.is_near_road(position, tolerance)
    }
    
    /// Get ground height at position
    /// 
    /// Pure function for ground level calculation.
    /// Simplified version that returns terrain baseline.
    pub fn get_ground_height(_position: Vec2) -> f32 {
        // Simple ground detection - matches terrain level
        -0.15
    }
    
    /// Clamp position to world bounds
    /// 
    /// Pure function that ensures position stays within valid world coordinates.
    pub fn clamp_to_world_bounds(position: Vec3, max_coord: f32) -> Vec3 {
        position.clamp(
            Vec3::splat(-max_coord),
            Vec3::splat(max_coord),
        )
    }
    
    /// Get collision tolerance distance for content type
    /// 
    /// Pure function returning minimum safe distance between entities.
    /// Used for collision detection and spawn validation.
    pub fn get_collision_tolerance(content_type: ContentType) -> f32 {
        match content_type {
            ContentType::Building => 35.0, // Buildings need significant space
            ContentType::Vehicle => 25.0,  // Vehicles need road clearance
            ContentType::Tree => 10.0,     // Trees can be closer together
            ContentType::NPC => 5.0,       // NPCs are small and mobile
            _ => 15.0,                     // Default safety distance
        }
    }
    
    /// Get road tolerance for content type
    /// 
    /// Pure function returning road proximity requirements.
    /// Negative values mean the content type REQUIRES roads.
    fn get_road_tolerance(content_type: ContentType) -> f32 {
        match content_type {
            ContentType::Building => 25.0,  // Buildings avoid roads
            ContentType::Tree => 15.0,      // Trees avoid roads
            ContentType::Vehicle => -8.0,   // Negative means vehicles NEED roads
            ContentType::NPC => 0.0,        // NPCs don't care about roads
            _ => 10.0,                      // Default avoidance distance
        }
    }
}

/// Advanced spawn validation utilities
/// 
/// Additional pure functions for complex spawn scenarios.
pub struct AdvancedSpawnValidation;

impl AdvancedSpawnValidation {
    /// Find safe spawn position near preferred location
    /// 
    /// Pure function that searches for valid spawn positions using spiral pattern.
    /// Returns None if no safe position found within search parameters.
    pub fn find_safe_spawn_position(
        preferred_position: Vec3,
        content_type: ContentType,
        max_world_coord: f32,
        road_network: Option<&RoadNetwork>,
        existing_content: &[(Vec3, ContentType, f32)],
        max_search_radius: f32,
        max_attempts: u32,
    ) -> Option<Vec3> {
        // First try the preferred position
        if SpawnValidation::is_spawn_position_valid(
            preferred_position, 
            content_type, 
            max_world_coord, 
            road_network
        ) && !SpawnValidation::has_content_collision(
            preferred_position, 
            content_type, 
            existing_content
        ) {
            return Some(preferred_position);
        }
        
        // Use spiral search pattern for better distribution
        for attempt in 0..max_attempts {
            let angle = (attempt as f32) * 2.39996; // Golden angle for even distribution
            let distance = (attempt as f32 / max_attempts as f32) * max_search_radius;
            
            let offset = Vec3::new(
                angle.cos() * distance,
                0.0,
                angle.sin() * distance,
            );
            
            let test_position = preferred_position + offset;
            
            // Validate position at ground level
            let ground_level_position = Vec3::new(
                test_position.x, 
                preferred_position.y, 
                test_position.z
            );
            
            if SpawnValidation::is_spawn_position_valid(
                ground_level_position, 
                content_type, 
                max_world_coord, 
                road_network
            ) && !SpawnValidation::has_content_collision(
                ground_level_position, 
                content_type, 
                existing_content
            ) {
                return Some(ground_level_position);
            }
        }
        
        None
    }
    
    /// Check collision between two specific entities
    /// 
    /// Pure function for precise entity-to-entity collision checking.
    /// Returns overlap distance if collision detected.
    pub fn check_entity_collision(
        pos1: Vec3,
        type1: ContentType, 
        radius1: f32,
        pos2: Vec3,
        type2: ContentType,
        radius2: f32,
    ) -> Option<f32> {
        let min_distance_1 = SpawnValidation::get_collision_tolerance(type1);
        let min_distance_2 = SpawnValidation::get_collision_tolerance(type2);
        let required_distance = (min_distance_1 + min_distance_2) / 2.0 + radius1 + radius2 + 2.0;
        
        let actual_distance = pos1.distance(pos2);
        if actual_distance < required_distance {
            Some(required_distance - actual_distance) // How much they overlap
        } else {
            None // No collision
        }
    }
    
    /// Validate multiple spawn positions at once
    /// 
    /// Pure function for batch validation of spawn positions.
    /// Returns list of valid positions from input set.
    pub fn validate_spawn_positions(
        positions: &[Vec3],
        content_type: ContentType,
        max_world_coord: f32,
        road_network: Option<&RoadNetwork>,
        existing_content: &[(Vec3, ContentType, f32)],
    ) -> Vec<Vec3> {
        positions
            .iter()
            .filter(|&&pos| {
                SpawnValidation::is_spawn_position_valid(
                    pos, 
                    content_type, 
                    max_world_coord, 
                    road_network
                ) && !SpawnValidation::has_content_collision(
                    pos, 
                    content_type, 
                    existing_content
                )
            })
            .copied()
            .collect()
    }
}

/// Spawn Validation Plugin
/// 
/// Lightweight plugin that provides pure stateless validation functions.
/// Initializes required resources for spawn validation systems.
pub struct SpawnValidationPlugin;

impl Plugin for SpawnValidationPlugin {
    fn build(&self, app: &mut App) {
        // Initialize SpawnRegistry resource for systems that need entity tracking
        app.init_resource::<crate::systems::spawn_validation::SpawnRegistry>();
        
        info!("🔧 SpawnValidationPlugin: Pure stateless validation functions available");
    }
}
