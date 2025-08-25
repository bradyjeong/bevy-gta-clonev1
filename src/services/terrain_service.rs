use bevy::prelude::*;

const TERRAIN_FALLBACK_HEIGHT: f32 = -0.15; // Compatible with existing ground detection
const TERRAIN_CHUNK_SIZE: f32 = 64.0; // Size of terrain chunks in world units

/// Service for terrain height queries and management
/// Phase 1: Flat fallback, designed for future heightmap integration
#[derive(Resource)]
pub struct TerrainService {
    pub enabled: bool,
    pub fallback_height: f32,
    pub chunk_size: f32,
}

impl Default for TerrainService {
    fn default() -> Self {
        Self {
            enabled: true,
            fallback_height: TERRAIN_FALLBACK_HEIGHT,
            chunk_size: TERRAIN_CHUNK_SIZE,
        }
    }
}

impl TerrainService {
    /// Create a new terrain service with custom settings
    pub fn new(fallback_height: f32, chunk_size: f32) -> Self {
        Self {
            enabled: true,
            fallback_height,
            chunk_size,
        }
    }
    
    /// Get terrain height at a specific world position
    /// Phase 1: Returns flat fallback height for compatibility
    pub fn height_at(&self, _x: f32, _z: f32) -> f32 {
        if !self.enabled {
            return self.fallback_height;
        }
        
        // Phase 1: Flat terrain for compatibility with existing ground detection
        // Future phases will implement actual heightmap queries here
        self.fallback_height
    }
    
    /// Get terrain height at a Vec2 position for convenience
    pub fn height_at_vec2(&self, position: Vec2) -> f32 {
        self.height_at(position.x, position.y)
    }
    
    /// Get spawn height for an entity (terrain height + entity height offset)
    pub fn get_spawn_height(&self, x: f32, z: f32, entity_height: f32) -> f32 {
        let terrain_height = self.height_at(x, z);
        // Place entity with its bottom at terrain level
        terrain_height + entity_height * 0.5
    }
    
    /// Check if terrain is available at a position (always true for flat fallback)
    pub fn has_terrain_at(&self, _x: f32, _z: f32) -> bool {
        self.enabled
    }
    
    /// Get the chunk coordinates for a world position
    pub fn world_to_chunk(&self, x: f32, z: f32) -> (i32, i32) {
        let chunk_x = (x / self.chunk_size).floor() as i32;
        let chunk_z = (z / self.chunk_size).floor() as i32;
        (chunk_x, chunk_z)
    }
    
    /// Get the world position of a chunk's origin
    pub fn chunk_to_world(&self, chunk_x: i32, chunk_z: i32) -> Vec2 {
        Vec2::new(
            chunk_x as f32 * self.chunk_size,
            chunk_z as f32 * self.chunk_size,
        )
    }
    
    /// Validate if a position is suitable for spawning (avoiding obstacles)
    /// Compatible with existing GroundDetectionService validation
    pub fn is_spawn_position_valid(&self, x: f32, z: f32) -> bool {
        // Simple validation - maintain compatibility with ground detection
        let distance_from_origin = (x * x + z * z).sqrt();
        distance_from_origin > 10.0 // Stay away from central area
    }
    
    /// Enable or disable terrain service
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Update fallback height (useful for different terrain levels)
    pub fn set_fallback_height(&mut self, height: f32) {
        self.fallback_height = height;
    }
}
