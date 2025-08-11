use serde::{Deserialize, Serialize};

/// Performance configuration values
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub culling: CullingConfig,
    pub lod: LODConfig,
    pub caching: CachingConfig,
    pub entity_limits: EntityLimitsConfig,
    pub timing: TimingConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CullingConfig {
    pub building_distance: f32,
    pub vehicle_distance: f32,
    pub npc_distance: f32,
    pub tree_distance: f32,
    pub effect_distance: f32,
    pub check_interval: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LODConfig {
    pub building_lod_distances: Vec<f32>,
    pub vehicle_lod_distances: Vec<f32>,
    pub npc_lod_distances: Vec<f32>,
    pub terrain_lod_distances: Vec<f32>,
    pub update_interval: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CachingConfig {
    pub distance_cache_size: usize,
    pub distance_cache_ttl: u32,
    pub mesh_cache_size: usize,
    pub texture_cache_size: usize,
    pub frame_cache_capacity: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityLimitsConfig {
    pub max_buildings: usize,
    pub max_vehicles: usize,
    pub max_npcs: usize,
    pub max_trees: usize,
    pub max_effects: usize,
    pub max_sounds: usize,
    pub spawn_percentage_threshold: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimingConfig {
    pub lod_update_interval: f32,
    pub culling_update_interval: f32,
    pub audio_cleanup_interval: f32,
    pub effect_update_interval: f32,
    pub physics_update_interval: f32,
    pub ai_update_interval: f32,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            culling: CullingConfig {
                building_distance: 300.0,
                vehicle_distance: 150.0,
                npc_distance: 100.0,
                tree_distance: 200.0,
                effect_distance: 50.0,
                check_interval: 0.5,
            },
            lod: LODConfig {
                building_lod_distances: vec![50.0, 150.0, 300.0, 500.0],
                vehicle_lod_distances: vec![50.0, 100.0, 125.0, 150.0],
                npc_lod_distances: vec![25.0, 50.0, 75.0, 100.0],
                terrain_lod_distances: vec![100.0, 250.0, 500.0, 1000.0],
                update_interval: 0.2,
            },
            caching: CachingConfig {
                distance_cache_size: 2048,
                distance_cache_ttl: 5,
                mesh_cache_size: 1024,
                texture_cache_size: 512,
                frame_cache_capacity: 256,
            },
            entity_limits: EntityLimitsConfig {
                max_buildings: 200,
                max_vehicles: 50,
                max_npcs: 20,
                max_trees: 100,
                max_effects: 30,
                max_sounds: 16,
                spawn_percentage_threshold: 0.8,
            },
            timing: TimingConfig {
                lod_update_interval: 0.2,
                culling_update_interval: 0.5,
                audio_cleanup_interval: 2.0,
                effect_update_interval: 0.1,
                physics_update_interval: 0.016, // 60 FPS physics
                ai_update_interval: 0.3,
            },
        }
    }
}
