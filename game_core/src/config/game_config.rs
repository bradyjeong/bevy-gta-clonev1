use bevy::prelude::*;
use bevy_rapier3d::prelude::Group;
use serde::{Deserialize, Serialize};

/// Main game configuration structure containing all data-driven values
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
#[derive(Default)]
pub struct GameConfig {
    pub spawn_rates: SpawnRatesConfig,
    pub entity_limits: EntityLimitsConfig,
    pub lod_distances: LodDistancesConfig,
    pub culling_distances: CullingDistancesConfig,
    pub update_intervals: UpdateIntervalsConfig,
    pub physics: PhysicsConfig,
    pub vehicle_physics: VehiclePhysicsConfig,
    pub world: WorldConfig,
    pub audio: AudioConfig,
    pub visual: VisualConfig,
    pub npc_behavior: NpcBehaviorConfig,
    pub performance: PerformanceConfig,
    pub vehicles: VehicleConfig,
    pub npc: NpcConfig,
    pub camera: CameraConfig,
    pub batching: BatchingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnRatesConfig {
    pub buildings: f32,
    pub vehicles: f32,
    pub trees: f32,
    pub npcs: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityLimitsConfig {
    pub buildings: usize,
    pub vehicles: usize,
    pub npcs: usize,
    pub trees: usize,
    pub particles: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodDistancesConfig {
    pub full: f32,
    pub medium: f32,
    pub low: f32,
    pub cull: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CullingDistancesConfig {
    pub buildings: f32,
    pub vehicles: f32,
    pub npcs: f32,
    pub vegetation: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIntervalsConfig {
    pub road_generation: f32,
    pub dynamic_content: f32,
    pub culling: f32,
    pub lod_update: f32,
    pub npc_close: f32,
    pub effect_update: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConfig {
    pub min_collider_size: f32,
    pub max_collider_size: f32,
    pub max_world_coord: f32,
    pub min_world_coord: f32,
    pub max_velocity: f32,
    pub max_angular_velocity: f32,
    pub min_mass: f32,
    pub max_mass: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub ground_friction: f32,
    pub rolling_resistance: f32,
    pub dt_clamp_min: f32,
    pub dt_clamp_max: f32,
    pub static_group: u32,
    pub vehicle_group: u32,
    pub character_group: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehiclePhysicsConfig {
    pub wing_deploy_speed: f32,
    pub downforce_multiplier: f32,
    pub cooling_rate: f32,
    pub exhaust_timer_threshold: f32,
    pub tire_temp_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    pub chunk_size: f32,
    pub streaming_radius: f32,
    pub lake_position: Vec3,
    pub lake_size: f32,
    pub lake_depth: f32,
    pub active_radius: f32,
    pub road_marking_height: f32,
    pub lod_distances: [f32; 3],
    pub building_density: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub update_timer_threshold: f32,
    pub wind_strength: f32,
    pub max_audio_distance: f32,
    pub fade_distance: f32,
    pub engine_volume: f32,
    pub master_volume: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualConfig {
    pub particle_sphere_radius: f32,
    pub exhaust_sphere_radius: f32,
    pub smoke_sphere_radius: f32,
    pub emissive_intensity: f32,
    pub transparency_alpha: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcBehaviorConfig {
    pub reaction_time_min: f32,
    pub reaction_time_max: f32,
    pub update_interval_range: (f32, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub frame_consistency_factor: f32,
    pub fps_smoothing_factor: f32,
    pub cache_max_entries: usize,
    pub cache_duration: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleConfig {
    pub basic_car: VehicleTypeConfig,
    pub super_car: VehicleTypeConfig,
    pub helicopter: VehicleTypeConfig,
    pub f16: VehicleTypeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleTypeConfig {
    pub body_size: Vec3,
    pub collider_size: Vec3,
    pub mass: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub default_color: Color,
    pub max_speed: f32,
    pub acceleration: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcConfig {
    pub default_height: f32,
    pub default_build: f32,
    pub walk_speed: f32,
    pub capsule_height: f32,
    pub capsule_radius: f32,
    pub update_intervals: NpcUpdateIntervals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcUpdateIntervals {
    pub close_interval: f32,
    pub close_distance: f32,
    pub far_distance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraConfig {
    pub distance: f32,
    pub height: f32,
    pub lerp_speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchingConfig {
    pub transform_batch_size: usize,
    pub visibility_batch_size: usize,
    pub physics_batch_size: usize,
    pub lod_batch_size: usize,
    pub max_processing_time_ms: f32,
    pub priority_boost_frames: u64,
    pub lod_distance_threshold: f32,
    pub cleanup_stale_flags: bool,
    pub cleanup_interval: f32,
    pub max_stale_frames: u32,
}


impl Default for SpawnRatesConfig {
    fn default() -> Self {
        Self {
            buildings: 0.08,
            vehicles: 0.04,
            trees: 0.05,
            npcs: 0.01,
        }
    }
}

impl Default for EntityLimitsConfig {
    fn default() -> Self {
        Self {
            buildings: 80,
            vehicles: 20,
            npcs: 2,
            trees: 100,
            particles: 50,
        }
    }
}

impl Default for LodDistancesConfig {
    fn default() -> Self {
        Self {
            full: 50.0,
            medium: 150.0,
            low: 300.0,
            cull: 500.0,
        }
    }
}

impl Default for CullingDistancesConfig {
    fn default() -> Self {
        Self {
            buildings: 300.0,
            vehicles: 150.0,
            npcs: 100.0,
            vegetation: 200.0,
        }
    }
}

impl Default for UpdateIntervalsConfig {
    fn default() -> Self {
        Self {
            road_generation: 0.5,
            dynamic_content: 2.0,
            culling: 0.5,
            lod_update: 0.1,
            npc_close: 0.05,
            effect_update: 0.05,
        }
    }
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            min_collider_size: 0.01,
            max_collider_size: 100.0,
            max_world_coord: 10000.0,
            min_world_coord: -10000.0,
            max_velocity: 500.0,
            max_angular_velocity: 50.0,
            min_mass: 0.1,
            max_mass: 100000.0,
            linear_damping: 0.05,
            angular_damping: 0.05,
            ground_friction: 0.7,
            rolling_resistance: 0.015,
            dt_clamp_min: 0.001,
            dt_clamp_max: 0.05,
            static_group: 0x0001,
            vehicle_group: 0x0002,
            character_group: 0x0004,
        }
    }
}

impl Default for VehiclePhysicsConfig {
    fn default() -> Self {
        Self {
            wing_deploy_speed: 150.0,
            downforce_multiplier: 0.05,
            cooling_rate: 0.05,
            exhaust_timer_threshold: 0.04,
            tire_temp_factor: 0.01,
        }
    }
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            chunk_size: 200.0,
            streaming_radius: 800.0,
            lake_position: Vec3::new(300.0, -2.0, 300.0),
            lake_size: 200.0,
            lake_depth: 10.0,
            active_radius: 100.0,
            road_marking_height: 0.01,
            lod_distances: [50.0, 150.0, 300.0],
            building_density: 0.08,
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            update_timer_threshold: 0.05,
            wind_strength: 0.05,
            max_audio_distance: 200.0,
            fade_distance: 150.0,
            engine_volume: 0.8,
            master_volume: 0.7,
        }
    }
}

impl Default for VisualConfig {
    fn default() -> Self {
        Self {
            particle_sphere_radius: 0.05,
            exhaust_sphere_radius: 0.08,
            smoke_sphere_radius: 0.04,
            emissive_intensity: 0.05,
            transparency_alpha: 0.3,
        }
    }
}

impl Default for NpcBehaviorConfig {
    fn default() -> Self {
        Self {
            reaction_time_min: 0.03,
            reaction_time_max: 0.12,
            update_interval_range: (0.05, 0.2),
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            frame_consistency_factor: 0.01,
            fps_smoothing_factor: 0.05,
            cache_max_entries: 2048,
            cache_duration: 5.0,
        }
    }
}

impl GameConfig {
    pub fn validate_and_clamp(&mut self) {
        // Add validation and clamping logic here if needed
        // For now, this is a placeholder to satisfy the interface
    }

    /// Load configuration from RON file
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = std::fs::read_to_string(path)?;
        let config: GameConfig = ron::from_str(&config_str)?;
        Ok(config)
    }

    /// Save configuration to RON file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let config_str = ron::to_string(self)?;
        std::fs::write(path, config_str)?;
        Ok(())
    }

    /// Validate configuration values
    pub fn validate(&mut self) {
        // Clamp spawn rates to valid ranges
        self.spawn_rates.buildings = self.spawn_rates.buildings.clamp(0.0, 1.0);
        self.spawn_rates.vehicles = self.spawn_rates.vehicles.clamp(0.0, 1.0);
        self.spawn_rates.trees = self.spawn_rates.trees.clamp(0.0, 1.0);
        self.spawn_rates.npcs = self.spawn_rates.npcs.clamp(0.0, 1.0);
        
        // Validate physics values
        self.physics.min_collider_size = self.physics.min_collider_size.max(0.001);
        self.physics.dt_clamp_min = self.physics.dt_clamp_min.clamp(0.001, 0.1);
        self.physics.dt_clamp_max = self.physics.dt_clamp_max.clamp(0.01, 0.5);
        
        // Validate distances
        self.lod_distances.full = self.lod_distances.full.max(1.0);
        self.lod_distances.medium = self.lod_distances.medium.max(self.lod_distances.full);
        self.lod_distances.low = self.lod_distances.low.max(self.lod_distances.medium);
        self.lod_distances.cull = self.lod_distances.cull.max(self.lod_distances.low);
        
        // Validate entity limits
        self.entity_limits.buildings = self.entity_limits.buildings.max(1);
        self.entity_limits.vehicles = self.entity_limits.vehicles.max(1);
        self.entity_limits.npcs = self.entity_limits.npcs.max(1);
        self.entity_limits.trees = self.entity_limits.trees.max(1);
    }
}

impl PhysicsConfig {
    /// Convert u32 group to Rapier Group
    #[must_use] pub fn static_group(&self) -> Group {
        Group::from_bits_truncate(self.static_group)
    }
    
    #[must_use] pub fn vehicle_group(&self) -> Group {
        Group::from_bits_truncate(self.vehicle_group)
    }
    
    #[must_use] pub fn character_group(&self) -> Group {
        Group::from_bits_truncate(self.character_group)
    }


}

impl Default for VehicleConfig {
    fn default() -> Self {
        Self {
            basic_car: VehicleTypeConfig {
                body_size: Vec3::new(4.0, 2.0, 8.0),
                collider_size: Vec3::new(3.8, 1.8, 7.8),
                mass: 1500.0,
                linear_damping: 0.1,
                angular_damping: 0.1,
                default_color: Color::srgb(0.8, 0.2, 0.2),
                max_speed: 180.0,
                acceleration: 15.0,
            },
            super_car: VehicleTypeConfig {
                body_size: Vec3::new(4.2, 1.8, 8.5),
                collider_size: Vec3::new(4.0, 1.6, 8.3),
                mass: 1800.0,
                linear_damping: 0.08,
                angular_damping: 0.08,
                default_color: Color::srgb(0.2, 0.8, 0.2),
                max_speed: 220.0,
                acceleration: 20.0,
            },
            helicopter: VehicleTypeConfig {
                body_size: Vec3::new(12.0, 3.0, 12.0),
                collider_size: Vec3::new(11.8, 2.8, 11.8),
                mass: 3000.0,
                linear_damping: 0.05,
                angular_damping: 0.05,
                default_color: Color::srgb(0.2, 0.2, 0.8),
                max_speed: 200.0,
                acceleration: 10.0,
            },
            f16: VehicleTypeConfig {
                body_size: Vec3::new(8.0, 2.0, 15.0),
                collider_size: Vec3::new(7.8, 1.8, 14.8),
                mass: 8000.0,
                linear_damping: 0.02,
                angular_damping: 0.02,
                default_color: Color::srgb(0.5, 0.5, 0.5),
                max_speed: 500.0,
                acceleration: 25.0,
            },
        }
    }
}

impl Default for NpcConfig {
    fn default() -> Self {
        Self {
            default_height: 1.8,
            default_build: 1.0,
            walk_speed: 1.5,
            capsule_height: 1.8,
            capsule_radius: 0.3,
            update_intervals: NpcUpdateIntervals::default(),
        }
    }
}

impl Default for NpcUpdateIntervals {
    fn default() -> Self {
        Self {
            close_interval: 0.05,
            close_distance: 50.0,
            far_distance: 200.0,
        }
    }
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            distance: 15.0,
            height: 8.0,
            lerp_speed: 0.1,
        }
    }
}

impl Default for BatchingConfig {
    fn default() -> Self {
        Self {
            transform_batch_size: 100,
            visibility_batch_size: 200,
            physics_batch_size: 50,
            lod_batch_size: 150,
            max_processing_time_ms: 3.0,
            priority_boost_frames: 3,
            lod_distance_threshold: 5.0,
            cleanup_stale_flags: true,
            cleanup_interval: 1.0,
            max_stale_frames: 60,
        }
    }
}
