use bevy::prelude::*;
use bevy_rapier3d::prelude::Group;
use crate::services::timing_service::{EntityTimerType, SystemType, TimingStats};

/// Core service trait for all services
#[allow(dead_code)]
pub trait Service: Send + Sync + 'static {
    fn service_name(&self) -> &'static str;
    fn is_ready(&self) -> bool { true }
}

/// Configuration service for centralized configuration management
#[allow(dead_code)]
pub trait ConfigService: Service {
    fn get_physics_config(&self) -> &crate::config::PhysicsConfig;
    fn get_world_config(&self) -> &crate::config::WorldConfig;
    fn get_vehicle_config(&self) -> &crate::config::VehicleConfig;
    fn get_npc_config(&self) -> &crate::config::NPCConfig;
    fn get_performance_config(&self) -> &crate::config::PerformanceConfig;
    fn get_audio_config(&self) -> &crate::config::AudioConfig;
    
    fn get_camera_config(&self) -> &crate::config::CameraConfig;
    fn get_ui_config(&self) -> &crate::config::UIConfig;
    
    fn update_config(&mut self, config: crate::config::GameConfig);
    fn validate_and_clamp(&mut self);
}

/// Timing service for throttling and timing management
#[allow(dead_code)]
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



/// Physics service for physics world interface
#[allow(dead_code)]
pub trait PhysicsService: Service {
    fn validate_position(&self, position: Vec3) -> Vec3;
    fn validate_velocity(&self, velocity: Vec3) -> Vec3;
    fn validate_mass(&self, mass: f32) -> f32;
    fn validate_collider_size(&self, size: Vec3) -> Vec3;
    fn get_collision_groups(&self) -> (Group, Group, Group); // static, vehicle, character
}


