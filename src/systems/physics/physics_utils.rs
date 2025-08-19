use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::config::GameConfig;
use crate::constants::{STATIC_GROUP, VEHICLE_GROUP, CHARACTER_GROUP};

/// Essential physics utilities for movement systems
#[derive(Default)]
pub struct PhysicsUtilities;

impl PhysicsUtilities {
    /// Validate and clamp velocity to safe ranges for physics stability
    /// Unified method that replaces both validate_velocity and apply_velocity_clamps
    pub fn clamp_velocity(velocity: &mut Velocity, config: &GameConfig) {
        // Clamp linear velocity to prevent physics instability
        velocity.linvel = velocity.linvel.clamp_length_max(config.physics.max_velocity);
        velocity.angvel = velocity.angvel.clamp_length_max(config.physics.max_angular_velocity);
        
        // Ensure all values are finite
        if !velocity.linvel.is_finite() {
            velocity.linvel = Vec3::ZERO;
        }
        if !velocity.angvel.is_finite() {
            velocity.angvel = Vec3::ZERO;
        }
    }
    
    /// Legacy alias for backward compatibility - use clamp_velocity instead
    #[deprecated(note = "Use clamp_velocity instead")]
    pub fn validate_velocity(velocity: &mut Velocity, config: &GameConfig) {
        Self::clamp_velocity(velocity, config);
    }
    

    
    /// Unified stable delta-time for all vehicle systems
    /// Prevents physics instability from frame rate spikes
    pub fn stable_dt(time: &Time) -> f32 {
        time.delta_secs().clamp(0.001, 0.05)
    }
    
    /// Clamp entity position to world bounds
    pub fn apply_world_bounds(
        transform: &mut Transform,
        velocity: &mut Velocity,
        config: &GameConfig
    ) {
        let bounds = config.physics.max_world_coord;
        
        // Check and clamp X bounds
        if transform.translation.x > bounds {
            transform.translation.x = bounds;
            velocity.linvel.x = velocity.linvel.x.min(0.0);
        } else if transform.translation.x < -bounds {
            transform.translation.x = -bounds;
            velocity.linvel.x = velocity.linvel.x.max(0.0);
        }
        
        // Check and clamp Z bounds
        if transform.translation.z > bounds {
            transform.translation.z = bounds;
            velocity.linvel.z = velocity.linvel.z.min(0.0);
        } else if transform.translation.z < -bounds {
            transform.translation.z = -bounds;
            velocity.linvel.z = velocity.linvel.z.max(0.0);
        }
    }
}

/// Collision group management utilities
pub struct CollisionGroupHelper;

impl CollisionGroupHelper {
    /// Get collision groups for static objects (buildings, terrain)
    pub fn static_groups() -> CollisionGroups {
        CollisionGroups::new(STATIC_GROUP, Group::ALL)
    }
    
    /// Get collision groups for vehicles (cars, aircraft)
    pub fn vehicle_groups() -> CollisionGroups {
        CollisionGroups::new(VEHICLE_GROUP, STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP)
    }
    
    /// Get collision groups for characters (player, NPCs)
    pub fn character_groups() -> CollisionGroups {
        CollisionGroups::new(CHARACTER_GROUP, STATIC_GROUP | VEHICLE_GROUP)
    }
}

/// Comprehensive physics safety system
pub fn apply_universal_physics_safeguards(
    mut query: Query<(Entity, &mut Velocity, &mut Transform), With<RigidBody>>,
    config: Res<GameConfig>,
) {
    for (_entity, mut velocity, mut transform) in query.iter_mut() {
        // Apply all safety measures
        PhysicsUtilities::clamp_velocity(&mut velocity, &config);
        PhysicsUtilities::apply_world_bounds(&mut transform, &mut velocity, &config);
        
        // Additional safety checks
        if !transform.translation.is_finite() {
            warn!("Entity had invalid position, resetting to origin");
            transform.translation = Vec3::ZERO;
            velocity.linvel = Vec3::ZERO;
            velocity.angvel = Vec3::ZERO;
        }
        
        if !transform.rotation.is_finite() {
            warn!("Entity had invalid rotation, resetting to identity");
            transform.rotation = Quat::IDENTITY;
            velocity.angvel = Vec3::ZERO;
        }
    }
}
