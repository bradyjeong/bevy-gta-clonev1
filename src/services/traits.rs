use bevy::prelude::*;
use bevy_rapier3d::prelude::Group;
use crate::systems::timing_service::{EntityTimerType, SystemType, TimingStats};

/// Core service trait for all services
pub trait Service: Send + Sync + 'static {
    fn service_name(&self) -> &'static str;
    fn is_ready(&self) -> bool { true }
}

/// Configuration service for centralized configuration management
pub trait ConfigService: Service {
    fn get_physics_config(&self) -> &crate::config::PhysicsConfig;
    fn get_world_config(&self) -> &crate::config::WorldConfig;
    fn get_vehicle_config(&self) -> &crate::config::VehicleConfig;
    fn get_npc_config(&self) -> &crate::config::NPCConfig;
    fn get_performance_config(&self) -> &crate::config::PerformanceConfig;
    fn get_audio_config(&self) -> &crate::config::AudioConfig;
    
    fn get_camera_config(&self) -> &crate::config::CameraConfig;
    fn get_ui_config(&self) -> &crate::config::UIConfig;
    fn get_batching_config(&self) -> &crate::config::BatchingConfig;
    
    fn update_config(&mut self, config: crate::config::GameConfig);
    fn validate_and_clamp(&mut self);
}

/// Timing service for throttling and timing management
pub trait TimingService: Service {
    fn current_time(&self) -> f32;
    fn delta_time(&self) -> f32;
    fn should_run_system(&mut self, system_type: SystemType) -> bool;
    fn register_entity(&mut self, entity: Entity, timer_type: EntityTimerType, interval: f32);
    fn should_update_entity(&mut self, entity: Entity) -> bool;
    fn unregister_entity(&mut self, entity: Entity);
    fn get_stats(&self) -> TimingStats;
    fn update_time(&mut self, time: &Time);
}

/// Audio service for audio system management
pub trait AudioService: Service {
    fn play_sound(&mut self, sound_id: &str, position: Option<Vec3>, volume: f32);
    fn stop_sound(&mut self, sound_id: &str);
    fn set_master_volume(&mut self, volume: f32);
    fn get_master_volume(&self) -> f32;
    fn cleanup_old_sounds(&mut self);
    fn update_spatial_audio(&mut self, listener_position: Vec3);
}

/// Asset service for managing meshes, materials, and other assets
pub trait AssetService: Service {
    fn get_mesh(&self, mesh_id: &str) -> Option<Handle<Mesh>>;
    fn get_material(&self, material_id: &str) -> Option<Handle<StandardMaterial>>;
    fn register_mesh(&mut self, mesh_id: String, handle: Handle<Mesh>);
    fn register_material(&mut self, material_id: String, handle: Handle<StandardMaterial>);
    fn cleanup_unused_assets(&mut self);
}

/// Physics service for physics world interface
pub trait PhysicsService: Service {
    fn validate_position(&self, position: Vec3) -> Vec3;
    fn validate_velocity(&self, velocity: Vec3) -> Vec3;
    fn validate_mass(&self, mass: f32) -> f32;
    fn validate_collider_size(&self, size: Vec3) -> Vec3;
    fn get_collision_groups(&self) -> (Group, Group, Group); // static, vehicle, character
}

/// Logging service for centralized logging
pub trait LoggingService: Service {
    fn log_info(&self, message: &str);
    fn log_warning(&self, message: &str);
    fn log_error(&self, message: &str);
    fn log_debug(&self, message: &str);
    fn set_log_level(&mut self, level: LogLevel);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}
