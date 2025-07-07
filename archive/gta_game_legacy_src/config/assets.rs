use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Vehicle configuration loaded from RON files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleConfig {
    pub max_speed: f32,
    pub acceleration: f32,
    pub deceleration: f32,
    pub turn_rate: f32,
    pub mass: f32,
    pub engine_power: f32,
    pub aerodynamic_drag: f32,
    pub downforce: f32,
}

/// Collection of all vehicle configurations
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct VehicleStats {
    pub vehicles: std::collections::HashMap<String, VehicleConfig>,
}

/// Performance and culling distances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CullingDistances {
    pub buildings: f32,
    pub vehicles: f32,
    pub npcs: f32,
    pub vegetation: f32,
    pub effects: f32,
}

/// LOD transition distances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodDistances {
    pub high_detail: f32,
    pub medium_detail: f32,
    pub sleep_mode: f32,
}

/// Entity spawn rates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnRates {
    pub buildings: f32,
    pub vehicles: f32,
    pub trees: f32,
    pub npcs: f32,
}

/// Performance targets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTargets {
    pub target_fps: f32,
    pub frame_time_budget_ms: f32,
    pub max_entities: usize,
    pub max_active_systems: usize,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSettings {
    pub distance_cache_size: usize,
    pub cache_duration_frames: u64,
    pub cleanup_interval_frames: u64,
}

/// Main performance configuration
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct PerformanceSettings {
    pub culling_distances: CullingDistances,
    pub lod_distances: LodDistances,
    pub spawn_rates: SpawnRates,
    pub performance_targets: PerformanceTargets,
    pub cache_settings: CacheSettings,
}

impl Default for VehicleConfig {
    fn default() -> Self {
        Self {
            max_speed: 50.0,
            acceleration: 15.0,
            deceleration: 20.0,
            turn_rate: 2.0,
            mass: 1500.0,
            engine_power: 200.0,
            aerodynamic_drag: 0.3,
            downforce: 0.1,
        }
    }
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            culling_distances: CullingDistances {
                buildings: 300.0,
                vehicles: 150.0,
                npcs: 100.0,
                vegetation: 200.0,
                effects: 80.0,
            },
            lod_distances: LodDistances {
                high_detail: 100.0,
                medium_detail: 200.0,
                sleep_mode: 300.0,
            },
            spawn_rates: SpawnRates {
                buildings: 0.08,
                vehicles: 0.04,
                trees: 0.05,
                npcs: 0.01,
            },
            performance_targets: PerformanceTargets {
                target_fps: 60.0,
                frame_time_budget_ms: 16.67,
                max_entities: 2000,
                max_active_systems: 50,
            },
            cache_settings: CacheSettings {
                distance_cache_size: 2048,
                cache_duration_frames: 5,
                cleanup_interval_frames: 300,
            },
        }
    }
}
