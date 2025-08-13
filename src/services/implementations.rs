use bevy::prelude::*;
use bevy_rapier3d::prelude::Group;
use std::collections::HashMap;
use crate::config::GameConfig;
use crate::systems::timing_service::{EntityTimerType, SystemType, TimingStats};
use super::traits::*;

/// Default configuration service implementation
pub struct DefaultConfigService {
    config: GameConfig,
}

impl DefaultConfigService {
    pub fn new(config: GameConfig) -> Self {
        Self { config }
    }
}

impl Service for DefaultConfigService {
    fn service_name(&self) -> &'static str {
        "ConfigService"
    }
}

impl ConfigService for DefaultConfigService {
    fn get_physics_config(&self) -> &crate::config::PhysicsConfig {
        &self.config.physics
    }
    
    fn get_world_config(&self) -> &crate::config::WorldConfig {
        &self.config.world
    }
    
    fn get_vehicle_config(&self) -> &crate::config::VehicleConfig {
        &self.config.vehicles
    }
    
    fn get_npc_config(&self) -> &crate::config::NPCConfig {
        &self.config.npc
    }
    
    fn get_performance_config(&self) -> &crate::config::PerformanceConfig {
        &self.config.performance
    }
    
    fn get_audio_config(&self) -> &crate::config::AudioConfig {
        &self.config.audio
    }
    
    
    
    fn get_camera_config(&self) -> &crate::config::CameraConfig {
        &self.config.camera
    }
    
    fn get_ui_config(&self) -> &crate::config::UIConfig {
        &self.config.ui
    }
    
    fn get_batching_config(&self) -> &crate::config::BatchingConfig {
        &self.config.batching
    }
    
    fn update_config(&mut self, config: GameConfig) {
        self.config = config;
    }
    
    fn validate_and_clamp(&mut self) {
        self.config.validate_and_clamp();
    }
}

/// Default timing service implementation (wraps existing TimingService)
pub struct DefaultTimingService {
    timing_service: crate::systems::timing_service::TimingService,
}

impl DefaultTimingService {
    pub fn new() -> Self {
        Self {
            timing_service: crate::systems::timing_service::TimingService::default(),
        }
    }
}

impl Service for DefaultTimingService {
    fn service_name(&self) -> &'static str {
        "TimingService"
    }
}

impl TimingService for DefaultTimingService {
    fn current_time(&self) -> f32 {
        self.timing_service.current_time
    }
    
    fn delta_time(&self) -> f32 {
        self.timing_service.delta_time
    }
    
    fn should_run_system(&mut self, system_type: SystemType) -> bool {
        self.timing_service.should_run_system(system_type)
    }
    
    fn register_entity(&mut self, entity: Entity, timer_type: EntityTimerType, interval: f32) {
        self.timing_service.register_entity(entity, timer_type, interval);
    }
    
    fn should_update_entity(&mut self, entity: Entity) -> bool {
        self.timing_service.should_update_entity(entity)
    }
    
    fn unregister_entity(&mut self, entity: Entity) {
        self.timing_service.unregister_entity(entity);
    }
    
    fn get_stats(&self) -> TimingStats {
        self.timing_service.get_stats()
    }
    
    fn update_time(&mut self, time: &Time) {
        self.timing_service.update(time);
    }
}

/// Default audio service implementation
pub struct DefaultAudioService {
    master_volume: f32,
    active_sounds: HashMap<String, f32>, // sound_id -> remaining_time
    spatial_sounds: HashMap<String, Vec3>, // sound_id -> position
}

impl DefaultAudioService {
    pub fn new() -> Self {
        Self {
            master_volume: 1.0,
            active_sounds: HashMap::new(),
            spatial_sounds: HashMap::new(),
        }
    }
}

impl Service for DefaultAudioService {
    fn service_name(&self) -> &'static str {
        "AudioService"
    }
}

impl AudioService for DefaultAudioService {
    fn play_sound(&mut self, sound_id: &str, position: Option<Vec3>, volume: f32) {
        let effective_volume = volume * self.master_volume;
        self.active_sounds.insert(sound_id.to_string(), 5.0); // Default 5 second duration
        
        if let Some(pos) = position {
            self.spatial_sounds.insert(sound_id.to_string(), pos);
        }
        
        // TODO: Integrate with actual audio system
        debug!("🔊 AUDIO: Playing {} at volume {:.2}", sound_id, effective_volume);
    }
    
    fn stop_sound(&mut self, sound_id: &str) {
        self.active_sounds.remove(sound_id);
        self.spatial_sounds.remove(sound_id);
        debug!("🔇 AUDIO: Stopped {}", sound_id);
    }
    
    fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 2.0);
    }
    
    fn get_master_volume(&self) -> f32 {
        self.master_volume
    }
    
    fn cleanup_old_sounds(&mut self) {
        let delta_time = 0.016; // Assume 60 FPS for now
        self.active_sounds.retain(|sound_id, remaining_time| {
            *remaining_time -= delta_time;
            if *remaining_time <= 0.0 {
                self.spatial_sounds.remove(sound_id);
                false
            } else {
                true
            }
        });
    }
    
    fn update_spatial_audio(&mut self, listener_position: Vec3) {
        // TODO: Update spatial audio based on listener position
        let sounds_to_stop: Vec<String> = self.spatial_sounds.iter()
            .filter_map(|(sound_id, sound_position)| {
                let distance = listener_position.distance(*sound_position);
                if distance > 250.0 { // Max audio distance
                    Some(sound_id.clone())
                } else {
                    None
                }
            })
            .collect();
        
        for sound_id in sounds_to_stop {
            self.stop_sound(&sound_id);
        }
    }
}

/// Default asset service implementation
pub struct DefaultAssetService {
    meshes: HashMap<String, Handle<Mesh>>,
    materials: HashMap<String, Handle<StandardMaterial>>,
}

impl DefaultAssetService {
    pub fn new() -> Self {
        Self {
            meshes: HashMap::new(),
            materials: HashMap::new(),
        }
    }
}

impl Service for DefaultAssetService {
    fn service_name(&self) -> &'static str {
        "AssetService"
    }
}

impl AssetService for DefaultAssetService {
    fn get_mesh(&self, mesh_id: &str) -> Option<Handle<Mesh>> {
        self.meshes.get(mesh_id).cloned()
    }
    
    fn get_material(&self, material_id: &str) -> Option<Handle<StandardMaterial>> {
        self.materials.get(material_id).cloned()
    }
    
    fn register_mesh(&mut self, mesh_id: String, handle: Handle<Mesh>) {
        self.meshes.insert(mesh_id, handle);
    }
    
    fn register_material(&mut self, material_id: String, handle: Handle<StandardMaterial>) {
        self.materials.insert(material_id, handle);
    }
    
    fn cleanup_unused_assets(&mut self) {
        // TODO: Implement asset cleanup based on reference counting
        debug!("🧹 ASSETS: Cleanup requested ({} meshes, {} materials)", 
               self.meshes.len(), self.materials.len());
    }
}

/// Default physics service implementation
pub struct DefaultPhysicsService {
    physics_config: crate::config::PhysicsConfig,
}

impl DefaultPhysicsService {
    pub fn new(physics_config: crate::config::PhysicsConfig) -> Self {
        Self { physics_config }
    }
}

impl Service for DefaultPhysicsService {
    fn service_name(&self) -> &'static str {
        "PhysicsService"
    }
}

impl PhysicsService for DefaultPhysicsService {
    fn validate_position(&self, position: Vec3) -> Vec3 {
        Vec3::new(
            position.x.clamp(self.physics_config.min_world_coord, self.physics_config.max_world_coord),
            position.y.clamp(self.physics_config.min_world_coord, self.physics_config.max_world_coord),
            position.z.clamp(self.physics_config.min_world_coord, self.physics_config.max_world_coord),
        )
    }
    
    fn validate_velocity(&self, velocity: Vec3) -> Vec3 {
        let speed = velocity.length();
        if speed > self.physics_config.max_velocity {
            velocity.normalize() * self.physics_config.max_velocity
        } else {
            velocity
        }
    }
    
    fn validate_mass(&self, mass: f32) -> f32 {
        mass.clamp(self.physics_config.min_mass, self.physics_config.max_mass)
    }
    
    fn validate_collider_size(&self, size: Vec3) -> Vec3 {
        Vec3::new(
            size.x.clamp(self.physics_config.min_collider_size, self.physics_config.max_collider_size),
            size.y.clamp(self.physics_config.min_collider_size, self.physics_config.max_collider_size),
            size.z.clamp(self.physics_config.min_collider_size, self.physics_config.max_collider_size),
        )
    }
    
    fn get_collision_groups(&self) -> (Group, Group, Group) {
        (
            self.physics_config.static_group,
            self.physics_config.vehicle_group,
            self.physics_config.character_group,
        )
    }
}

/// Default logging service implementation
pub struct DefaultLoggingService {
    log_level: LogLevel,
}

impl DefaultLoggingService {
    pub fn new() -> Self {
        Self {
            log_level: LogLevel::Info,
        }
    }
}

impl Service for DefaultLoggingService {
    fn service_name(&self) -> &'static str {
        "LoggingService"
    }
}

impl LoggingService for DefaultLoggingService {
    fn log_info(&self, message: &str) {
        if self.log_level <= LogLevel::Info {
            info!("{}", message);
        }
    }
    
    fn log_warning(&self, message: &str) {
        if self.log_level <= LogLevel::Warning {
            warn!("{}", message);
        }
    }
    
    fn log_error(&self, message: &str) {
        if self.log_level <= LogLevel::Error {
            error!("{}", message);
        }
    }
    
    fn log_debug(&self, message: &str) {
        if self.log_level <= LogLevel::Debug {
            debug!("{}", message);
        }
    }
    
    fn set_log_level(&mut self, level: LogLevel) {
        self.log_level = level;
    }
}

impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some((*self as u8).cmp(&(*other as u8)))
    }
}
