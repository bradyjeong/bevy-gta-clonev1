use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityLimits {
    pub buildings: u32,
    pub vehicles: u32,
    pub npcs: u32,
    pub vegetation: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnRates {
    pub buildings: f32,
    pub vehicles: f32,
    pub trees: f32,
    pub npcs: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CullingDistances {
    pub buildings: f32,
    pub vehicles: f32,
    pub npcs: f32,
    pub vegetation: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIntervals {
    pub road_generation: f32,
    pub dynamic_content: f32,
    pub culling: f32,
    pub lod_update: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSettings {
    pub max_entries: usize,
    pub cache_duration: f32,
    pub cleanup_interval: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct PerformanceConfig {
    pub target_fps: f32,
    pub max_entities: EntityLimits,
    pub spawn_rates: SpawnRates,
    pub culling_distances: CullingDistances,
    pub update_intervals: UpdateIntervals,
    pub cache_settings: CacheSettings,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            target_fps: 60.0,
            max_entities: EntityLimits {
                buildings: 500,
                vehicles: 50,
                npcs: 30,
                vegetation: 1000,
            },
            spawn_rates: SpawnRates {
                buildings: 0.08,
                vehicles: 0.04,
                trees: 0.05,
                npcs: 0.01,
            },
            culling_distances: CullingDistances {
                buildings: 300.0,
                vehicles: 150.0,
                npcs: 100.0,
                vegetation: 200.0,
            },
            update_intervals: UpdateIntervals {
                road_generation: 0.5,
                dynamic_content: 2.0,
                culling: 0.5,
                lod_update: 0.1,
            },
            cache_settings: CacheSettings {
                max_entries: 2048,
                cache_duration: 5.0,
                cleanup_interval: 10.0,
            },
        }
    }
}

/// Performance monitoring counters as Resource
#[derive(Debug, Clone, Default, Resource)]
pub struct PerformanceCounters {
    pub frame_count: u64,
    pub entity_count: u32,
    pub culled_entities: u32,
    pub lod_updates: u32,
    pub cache_hits: u32,
    pub cache_misses: u32,
    pub last_fps: f32,
    pub avg_frame_time: f32,
    pub last_update: f32,
}

impl PerformanceCounters {
    pub fn update_frame(&mut self, delta_time: f32) {
        self.frame_count += 1;
        self.avg_frame_time = self.avg_frame_time * 0.95 + delta_time * 0.05;
        self.last_fps = 1.0 / delta_time;
        self.last_update = delta_time;
    }
    
    pub fn reset_per_frame_counters(&mut self) {
        self.lod_updates = 0;
        self.culled_entities = 0;
    }
}
