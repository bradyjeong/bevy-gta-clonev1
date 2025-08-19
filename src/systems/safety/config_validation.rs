use bevy::prelude::*;
use crate::config::GameConfig;
use crate::components::safety::WorldBounds;

/// Startup validation to prevent physics explosions
pub fn validate_physics_config(
    config: Res<GameConfig>,
    bounds: Res<WorldBounds>,
) {
    // Critical: max_velocity must be reasonable to prevent coordinate explosion
    let max_vel = config.physics.max_velocity;
    let max_coord = bounds.max_coordinate;
    
    if max_vel > max_coord / 2.0 {
        panic!("Physics config error: max_velocity ({}) exceeds half of world bounds ({}). This can cause coordinate explosion.", max_vel, max_coord / 2.0);
    }
    
    if max_vel > 1500.0 {
        warn!("Physics config: max_velocity ({}) is very high (>Mach 4.5). Consider reducing to prevent instability.", max_vel);
    }
    
    // Check for reasonable angular velocity limits
    let max_ang_vel = config.physics.max_angular_velocity;
    if max_ang_vel > 100.0 {
        warn!("Physics config: max_angular_velocity ({}) is very high. Consider reducing to prevent rotation chaos.", max_ang_vel);
    }
    
    info!("Physics config validated: max_velocity={}, max_angular_velocity={}, world_bounds={}", 
          max_vel, max_ang_vel, max_coord);
}
