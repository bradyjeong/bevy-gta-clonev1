//! Spatial components for game entities

use bevy::prelude::*;

/// Distance culling configuration for different entity types
#[derive(Debug, Clone)]
pub struct DistanceCullingConfig {
    /// Distance thresholds for different LOD levels
    pub lod_distances: Vec<f32>,
    /// Maximum distance before entity is completely culled
    pub cull_distance: f32,
    /// Hysteresis buffer to prevent flickering (applied to all distances)
    pub hysteresis: f32,
    /// How often to check distance (in seconds)
    pub update_interval: f32,
    /// Entity type identifier for debugging
    pub entity_type: &'static str,
}

/// Entity categories for distance culling
#[derive(Debug, Clone, PartialEq)]
pub enum CullingCategory {
    Building,
    Vehicle,
    NPC,
    Vegetation,
    Effect,
}

impl DistanceCullingConfig {
    /// Create config optimized for vehicles
    pub fn vehicle() -> Self {
        Self {
            lod_distances: vec![50.0, 150.0, 300.0], // Full, Medium, Low LOD
            cull_distance: 500.0,
            hysteresis: 5.0,
            update_interval: 0.5,
            entity_type: "Vehicle",
        }
    }

    /// Create config optimized for NPCs
    pub fn npc() -> Self {
        Self {
            lod_distances: vec![25.0, 75.0, 100.0], // Full, Medium, Low LOD
            cull_distance: 150.0,
            hysteresis: 3.0,
            update_interval: 0.3,
            entity_type: "NPC",
        }
    }

    /// Create config optimized for vegetation
    pub fn vegetation() -> Self {
        Self {
            lod_distances: vec![50.0, 150.0, 300.0], // Full, Medium, Billboard
            cull_distance: 400.0,
            hysteresis: 10.0,
            update_interval: 1.0, // Less frequent updates for vegetation
            entity_type: "Vegetation",
        }
    }

    /// Create config optimized for buildings
    pub fn building() -> Self {
        Self {
            lod_distances: vec![100.0, 300.0, 500.0], // Buildings visible at longer distances
            cull_distance: 800.0,
            hysteresis: 15.0,
            update_interval: 0.8,
            entity_type: "Building",
        }
    }

    /// Alias for building() for backwards compatibility
    pub fn buildings() -> Self {
        Self::building()
    }

    /// Create config optimized for effects
    pub fn effect() -> Self {
        Self {
            lod_distances: vec![30.0, 60.0, 120.0],
            cull_distance: 120.0,
            hysteresis: 2.0,
            update_interval: 0.2,
            entity_type: "Effect",
        }
    }

    /// Create config optimized for map chunks
    pub fn chunk() -> Self {
        Self {
            lod_distances: vec![150.0, 300.0, 500.0],
            cull_distance: 800.0,
            hysteresis: 20.0,
            update_interval: 0.5,
            entity_type: "Chunk",
        }
    }

    /// Alias for chunk() for backwards compatibility
    pub fn chunks() -> Self {
        Self::chunk()
    }

    /// Get LOD level for given distance
    pub fn get_lod_level(&self, distance: f32) -> usize {
        for (level, &threshold) in self.lod_distances.iter().enumerate() {
            if distance <= threshold + self.hysteresis {
                return level;
            }
        }
        self.lod_distances.len() // Beyond all LOD levels
    }

    /// Check if entity should be culled
    pub fn should_cull(&self, distance: f32) -> bool {
        distance > self.cull_distance + self.hysteresis
    }
}

/// Component to mark entities that use the unified culling system
#[derive(Component)]
pub struct UnifiedCullable {
    pub config: DistanceCullingConfig,
    pub current_lod: usize,
    pub is_culled: bool,
    pub last_distance: f32,
    pub last_update: f32,
}

impl UnifiedCullable {
    pub fn new(config: DistanceCullingConfig) -> Self {
        Self {
            config,
            current_lod: 0,
            is_culled: false,
            last_distance: 0.0,
            last_update: 0.0,
        }
    }

    pub fn vehicle() -> Self {
        Self::new(DistanceCullingConfig::vehicle())
    }

    pub fn npc() -> Self {
        Self::new(DistanceCullingConfig::npc())
    }

    pub fn building() -> Self {
        Self::new(DistanceCullingConfig::building())
    }

    pub fn vegetation() -> Self {
        Self::new(DistanceCullingConfig::vegetation())
    }

    pub fn effect() -> Self {
        Self::new(DistanceCullingConfig::effect())
    }

    pub fn chunk() -> Self {
        Self::new(DistanceCullingConfig::chunk())
    }

    pub fn should_cull(&self, distance: f32) -> bool {
        distance > self.config.cull_distance
    }

    pub fn get_lod_level(&self, distance: f32) -> usize {
        for (i, &lod_distance) in self.config.lod_distances.iter().enumerate() {
            if distance <= lod_distance {
                return i;
            }
        }
        self.config.lod_distances.len()
    }

    pub fn update_distance(&mut self, distance: f32, time: f32) {
        self.last_distance = distance;
        self.last_update = time;
        self.current_lod = self.get_lod_level(distance);
        self.is_culled = self.should_cull(distance);
    }

    /// Check if this entity needs an update based on time and distance change
    pub fn needs_update(&self, current_time: f32, current_distance: f32) -> bool {
        let time_elapsed = current_time - self.last_update;
        let distance_changed = (current_distance - self.last_distance).abs() > self.config.hysteresis;
        
        time_elapsed >= self.config.update_interval || distance_changed
    }
}

/// Component to track entities that have moved significantly
#[derive(Component)]
pub struct MovementTracker {
    pub last_position: Vec3,
    pub movement_threshold: f32,
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

/// Chunk coordinate for world streaming
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub fn from_world_position(position: Vec3, chunk_size: f32) -> Self {
        Self {
            x: (position.x / chunk_size).floor() as i32,
            z: (position.z / chunk_size).floor() as i32,
        }
    }

    pub fn from_world_pos(world_pos: Vec3, chunk_size: f32) -> Self {
        Self::from_world_position(world_pos, chunk_size)
    }

    pub fn to_world_pos(&self, chunk_size: f32) -> Vec3 {
        Vec3::new(
            self.x as f32 * chunk_size + chunk_size * 0.5,
            0.0,
            self.z as f32 * chunk_size + chunk_size * 0.5,
        )
    }
    
    pub fn distance_to(&self, other: ChunkCoord) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dz = (self.z - other.z) as f32;
        (dx * dx + dz * dz).sqrt()
    }
}

/// Component to mark entities that belong to specific world chunks
#[derive(Component)]
pub struct UnifiedChunkEntity {
    pub coord: ChunkCoord,
    pub layer: u32,
}
