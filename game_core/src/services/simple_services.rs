use bevy::prelude::*;
use crate::config::{GameConfig, game_config::PhysicsConfig};

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

/// System to initialize simple services
pub fn initialize_simple_services(
    mut commands: Commands,
    config: Res<GameConfig>,
) {
    // Register simple services as Bevy resources
    commands.insert_resource(ConfigService::new(config.clone()));
    commands.insert_resource(PhysicsService::new(config.physics.clone()));
    
    info!("✅ SIMPLE SERVICES: Initialized config and physics services");
}
