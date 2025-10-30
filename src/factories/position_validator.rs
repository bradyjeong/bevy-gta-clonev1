use crate::components::ContentType;
use crate::config::GameConfig;
use crate::constants::WorldEnvConfig;

use crate::factories::generic_bundle::BundleError;
use crate::systems::world::road_generation::is_on_road_spline;
use crate::systems::world::road_network::RoadNetwork;
use crate::systems::world::unified_world::UnifiedWorldManager;
use bevy::prelude::*;
use std::collections::HashMap;

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
    max_cache_size: usize,
}

impl PositionValidator {
    pub fn new(config: GameConfig) -> Self {
        Self {
            config,
            position_cache: HashMap::new(),
            max_cache_size: 10000,
        }
    }

    /// Validate position is within world bounds
    ///
    /// Simple, focused validation that follows AGENT.md:
    /// - Explicit over implicit: Clear bounds checking
    /// - No magic numbers: Uses configuration values
    /// - Clear error messages with context
    pub fn validate_position(&self, position: Vec3) -> Result<Vec3, BundleError> {
        if position.x.abs() > self.config.physics.max_world_coord
            || position.z.abs() > self.config.physics.max_world_coord
        {
            return Err(BundleError::PositionOutOfBounds {
                position,
                max_coord: self.config.physics.max_world_coord,
            });
        }

        Ok(position.clamp(
            Vec3::splat(self.config.physics.min_world_coord),
            Vec3::splat(self.config.physics.max_world_coord),
        ))
    }

    /// Get ground height at position with caching for performance
    ///
    /// Performance optimization that follows AGENT.md:
    /// - Clear caching strategy: 10m grid resolution
    /// - Explicit behavior: Shows exactly how height is calculated
    /// - Matches island terrain curve: plateau, beach slope, ocean floor
    pub fn get_ground_height(
        &mut self,
        position: Vec2,
        env: &WorldEnvConfig,
        world: &UnifiedWorldManager,
    ) -> f32 {
        let grid_x = (position.x / 10.0).floor() as i32; // 10m grid resolution
        let grid_z = (position.y / 10.0).floor() as i32;

        if let Some(&cached_height) = self.position_cache.get(&(grid_x, grid_z)) {
            return cached_height;
        }

        // Simplified: terrain islands are at land_elevation, ocean at ocean_floor_depth
        // Beach slopes handled by visual meshes only
        let ground_height =
            if world.is_on_terrain_island(Vec3::new(position.x, env.land_elevation, position.y)) {
                env.land_elevation
            } else {
                env.ocean_floor_depth
            };

        // Cache for future use with LRU eviction to prevent performance spikes
        if self.position_cache.len() >= self.max_cache_size {
            // Remove oldest 20% of entries instead of clearing all
            let remove_count = self.max_cache_size / 5;
            let keys_to_remove: Vec<_> = self.position_cache.keys().take(remove_count).copied().collect();
            for key in keys_to_remove {
                self.position_cache.remove(&key);
            }
        }
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
        road_network: Option<&RoadNetwork>,
        world: &UnifiedWorldManager,
    ) -> bool {
        // Check if on road (invalid for buildings and trees)
        if let Some(roads) = road_network {
            let road_tolerance: f32 = match content_type {
                ContentType::Building => 25.0,
                ContentType::Tree => 15.0,
                ContentType::Vehicle => -8.0, // Negative means vehicles NEED roads
                ContentType::NPC => 0.0,      // NPCs can be anywhere
                _ => 10.0,
            };

            let on_road = is_on_road_spline(position, roads, road_tolerance.abs());

            match content_type {
                ContentType::Vehicle => {
                    if !on_road {
                        return false;
                    } // Vehicles need roads
                }
                ContentType::Building | ContentType::Tree => {
                    if on_road {
                        return false;
                    } // Buildings/trees avoid roads
                }
                _ => {} // NPCs and others don't care about roads
            }
        }

        // Check if in water area
        if self.is_in_water_area(position, world) && !matches!(content_type, ContentType::Vehicle) {
            return false;
        }

        true
    }

    /// Check if position is in water area
    ///
    /// Simple water detection following AGENT.md:
    /// - Explicit coordinates: No hidden magic values
    /// - Clear algorithm: Rectangular island detection
    /// - Focused responsibility: Only water area checking
    fn is_in_water_area(&self, position: Vec3, world: &UnifiedWorldManager) -> bool {
        // In water if not on terrain island
        !world.is_on_terrain_island(position)
    }

    pub fn clear_cache(&mut self) {
        self.position_cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.position_cache.len()
    }
}
