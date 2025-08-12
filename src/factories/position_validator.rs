use bevy::prelude::*;
use std::collections::HashMap;
use crate::config::GameConfig;
use crate::components::ContentType;
use crate::factories::generic_bundle::BundleError;
use crate::world::RoadNetwork;
// Direct import allowed - factories and road_generation are both in UnifiedWorldPlugin
use crate::systems::world::road_generation::is_on_road_spline;

/// Position validator for entity spawning
/// 
/// Follows AGENT.md principles:
/// - Single responsibility: Validates positions for spawning
/// - Clear boundaries: Only handles position validation logic
/// - Minimal coupling: Only depends on config and basic types
/// - Straightforward data flow: Position in → validation checks → result out
#[derive(Debug)]
pub struct PositionValidator {
    config: GameConfig,
    position_cache: HashMap<(i32, i32), f32>,
}

impl PositionValidator {
    pub fn new(config: GameConfig) -> Self {
        Self {
            config,
            position_cache: HashMap::new(),
        }
    }
    
    /// Validate position is within world bounds
    /// 
    /// Simple, focused validation that follows AGENT.md:
    /// - Explicit over implicit: Clear bounds checking
    /// - No magic numbers: Uses configuration values
    /// - Clear error messages with context
    pub fn validate_position(&self, position: Vec3) -> Result<Vec3, BundleError> {
        if position.x.abs() > self.config.gameplay.physics.max_world_coord ||
           position.z.abs() > self.config.gameplay.physics.max_world_coord {
            return Err(BundleError::PositionOutOfBounds {
                position,
                max_coord: self.config.gameplay.physics.max_world_coord,
            });
        }
        
        Ok(position.clamp(
            Vec3::splat(self.config.gameplay.physics.min_world_coord),
            Vec3::splat(self.config.gameplay.physics.max_world_coord),
        ))
    }
    
    /// Get ground height at position with caching for performance
    /// 
    /// Performance optimization that follows AGENT.md:
    /// - Clear caching strategy: 10m grid resolution
    /// - Explicit behavior: Shows exactly how height is calculated
    /// - Simple algorithm: No complex terrain interpolation
    pub fn get_ground_height(&mut self, position: Vec2) -> f32 {
        let grid_x = (position.x / 10.0) as i32; // 10m grid resolution
        let grid_z = (position.y / 10.0) as i32;
        
        if let Some(&cached_height) = self.position_cache.get(&(grid_x, grid_z)) {
            return cached_height;
        }
        
        // Simple ground detection - would be enhanced with actual terrain data
        let ground_height = -0.15; // Match terrain level at y = -0.15
        
        // Cache for future use (following AGENT.md performance guidelines)
        self.position_cache.insert((grid_x, grid_z), ground_height);
        ground_height
    }
    
    /// Check if position is valid for spawning (not on roads, water, etc.)
    /// 
    /// Content-aware validation following AGENT.md principles:
    /// - Single purpose: Determine if position is suitable for content type
    /// - Explicit rules: Clear logic for each content type
    /// - No tangled dependencies: Self-contained validation logic
    pub fn is_spawn_position_valid(
        &self, 
        position: Vec3, 
        content_type: ContentType,
        road_network: Option<&RoadNetwork>
    ) -> bool {
        // Check if on road (invalid for buildings and trees)
        if let Some(roads) = road_network {
            let road_tolerance: f32 = match content_type {
                ContentType::Building => 25.0,
                ContentType::Tree => 15.0,
                ContentType::Vehicle => -8.0, // Negative means vehicles NEED roads
                ContentType::NPC => 0.0, // NPCs can be anywhere
                _ => 10.0,
            };
            
            let on_road = is_on_road_spline(position, roads, road_tolerance.abs());
            
            match content_type {
                ContentType::Vehicle => {
                    if !on_road { return false; } // Vehicles need roads
                }
                ContentType::Building | ContentType::Tree => {
                    if on_road { return false; } // Buildings/trees avoid roads
                }
                _ => {} // NPCs and others don't care about roads
            }
        }
        
        // Check if in water area
        if self.is_in_water_area(position) && !matches!(content_type, ContentType::Vehicle) {
            return false;
        }
        
        true
    }
    
    /// Check if position is in water area
    /// 
    /// Simple water detection following AGENT.md:
    /// - Explicit coordinates: No hidden magic values
    /// - Clear algorithm: Distance-based detection
    /// - Focused responsibility: Only water area checking
    fn is_in_water_area(&self, position: Vec3) -> bool {
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
}


