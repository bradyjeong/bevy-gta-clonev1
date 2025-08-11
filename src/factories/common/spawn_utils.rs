//! Shared spawn validation and positioning utilities
//! 
//! Stateless helper functions following AGENT.md simplicity principles

use bevy::prelude::*;
use std::collections::HashMap;
use crate::components::ContentType;
use crate::systems::RoadNetwork;
use crate::systems::is_on_road_spline;

/// Shared ground height calculation with caching
pub struct GroundHeightCache {
    cache: HashMap<(i32, i32), f32>,
}

impl Default for GroundHeightCache {
    fn default() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

impl GroundHeightCache {
    /// Get ground height at position with 10m grid caching
    pub fn get_ground_height(&mut self, position: Vec2) -> f32 {
        let grid_x = (position.x / 10.0) as i32;
        let grid_z = (position.y / 10.0) as i32;
        
        if let Some(&cached_height) = self.cache.get(&(grid_x, grid_z)) {
            return cached_height;
        }
        
        // Simple ground detection - matches terrain level
        let ground_height = -0.15;
        
        // Cache for future use
        self.cache.insert((grid_x, grid_z), ground_height);
        ground_height
    }
}

/// Spawn position validation utilities
pub struct SpawnValidation;

impl SpawnValidation {
    /// Check if position is valid for spawning this content type
    pub fn is_position_valid(
        position: Vec3,
        content_type: ContentType,
        road_network: Option<&RoadNetwork>,
    ) -> bool {
        // Check road constraints
        if let Some(roads) = road_network {
            let road_tolerance = Self::get_road_tolerance(content_type);
            let on_road = is_on_road_spline(position, roads, road_tolerance.abs());
            
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
        
        // Check water areas
        if Self::is_in_water_area(position) && !matches!(content_type, ContentType::Vehicle) {
            return false;
        }
        
        true
    }
    
    /// Get collision tolerance for content type
    pub fn get_collision_tolerance(content_type: ContentType) -> f32 {
        match content_type {
            ContentType::Building => 35.0,
            ContentType::Vehicle => 25.0,
            ContentType::Tree => 10.0,
            ContentType::NPC => 5.0,
            _ => 15.0,
        }
    }
    
    /// Get road tolerance for content type  
    fn get_road_tolerance(content_type: ContentType) -> f32 {
        match content_type {
            ContentType::Building => 25.0,
            ContentType::Tree => 15.0,
            ContentType::Vehicle => -8.0, // Negative means vehicles NEED roads
            ContentType::NPC => 0.0,
            _ => 10.0,
        }
    }
    
    /// Check if position is in water area
    fn is_in_water_area(position: Vec3) -> bool {
        // Lake position and size (must match water.rs setup)
        let lake_center = Vec3::new(300.0, -2.0, 300.0);
        let lake_size = 200.0;
        let buffer = 20.0;
        
        let distance = Vec2::new(
            position.x - lake_center.x,
            position.z - lake_center.z,
        ).length();
        
        distance < (lake_size / 2.0 + buffer)
    }
}
