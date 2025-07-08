use bevy::prelude::*;
use tracing::info;
use game_core::prelude::*;
use game_core::config::game_config::PhysicsConfig;
use crate::systems::timing_service::TimingService as BaseTimingService;

/// Simplified service injection using direct Bevy resources
/// This provides the service pattern while avoiding complex trait objects

/// Configuration service resource - wraps GameConfig
#[derive(Resource)]
pub struct ConfigService {
    config: GameConfig,
}

impl ConfigService {
    pub fn new(config: GameConfig) -> Self {
        Self { config }
    }
    
    pub fn get_config(&self) -> &GameConfig {
        &self.config
    }
    
    pub fn update_config(&mut self, config: GameConfig) {
        self.config = config;
    }
    
    pub fn validate_and_clamp(&mut self) {
        self.config.validate_and_clamp();
    }
}

/// Physics service resource
#[derive(Resource)]
pub struct PhysicsService {
    physics_config: PhysicsConfig,
}

impl PhysicsService {
    pub fn new(physics_config: PhysicsConfig) -> Self {
        Self { physics_config }
    }
    
    pub fn validate_position(&self, position: Vec3) -> Vec3 {
        Vec3::new(
            position.x.clamp(self.physics_config.min_world_coord, self.physics_config.max_world_coord),
            position.y.clamp(self.physics_config.min_world_coord, self.physics_config.max_world_coord),
            position.z.clamp(self.physics_config.min_world_coord, self.physics_config.max_world_coord),
        )
    }
    
    pub fn validate_velocity(&self, velocity: Vec3) -> Vec3 {
        let speed = velocity.length();
        if speed > self.physics_config.max_velocity {
            velocity.normalize() * self.physics_config.max_velocity
        } else {
            velocity
        }
    }
    
    pub fn validate_mass(&self, mass: f32) -> f32 {
        mass.clamp(self.physics_config.min_mass, self.physics_config.max_mass)
    }
    
    pub fn validate_collider_size(&self, size: Vec3) -> Vec3 {
        Vec3::new(
            size.x.clamp(self.physics_config.min_collider_size, self.physics_config.max_collider_size),
            size.y.clamp(self.physics_config.min_collider_size, self.physics_config.max_collider_size),
            size.z.clamp(self.physics_config.min_collider_size, self.physics_config.max_collider_size),
        )
    }
    
    pub fn get_collision_groups(&self) -> (bevy_rapier3d::prelude::Group, bevy_rapier3d::prelude::Group, bevy_rapier3d::prelude::Group) {
        (
            self.physics_config.static_group(),
            self.physics_config.vehicle_group(),
            self.physics_config.character_group(),
        )
    }
}

/// Enhanced timing service resource (wraps existing TimingService)
#[derive(Resource)]
pub struct EnhancedTimingService {
    base_service: BaseTimingService,
}

impl EnhancedTimingService {
    pub fn new() -> Self {
        Self {
            base_service: BaseTimingService::default(),
        }
    }
    
    pub fn update_time(&mut self, time: &Time) {
        self.base_service.update(time);
    }
    
    pub fn should_run_system(&mut self, system_type: crate::systems::timing_service::SystemType) -> bool {
        self.base_service.should_run_system(system_type)
    }
    
    pub fn current_time(&self) -> f32 {
        self.base_service.current_time()
    }
    
    pub fn delta_time(&self) -> f32 {
        self.base_service.delta_time()
    }
}

/// System to initialize simple services
pub fn initialize_simple_services(
    mut commands: Commands,
    config: Res<GameConfig>,
) {
    // Register simple services as Bevy resources
    commands.insert_resource(ConfigService::new(config.clone()));
    commands.insert_resource(PhysicsService::new(config.physics.clone()));
    commands.insert_resource(EnhancedTimingService::new());
    
    info!("✅ SIMPLE SERVICES: Initialized config, physics, and timing services");
}

/// System to update timing service
pub fn update_timing_service_system(
    mut timing_service: ResMut<EnhancedTimingService>,
    time: Res<Time>,
) {
    timing_service.update_time(&time);
}
