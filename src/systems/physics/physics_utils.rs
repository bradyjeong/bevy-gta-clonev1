use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::config::GameConfig;
use crate::constants::{STATIC_GROUP, VEHICLE_GROUP, CHARACTER_GROUP};
use crate::util::safe_math::{sanitize_velocity, sanitize_transform};

/// Essential physics utilities for movement systems
#[derive(Default)]
pub struct PhysicsUtilities;

impl PhysicsUtilities {
    /// Validate and clamp velocity to safe ranges for physics stability
    /// Enhanced with safe math to prevent all corruption sources
    pub fn clamp_velocity(velocity: &mut Velocity, config: &GameConfig) {
        // Use safe math utilities for comprehensive validation
        let was_corrupt = sanitize_velocity(velocity);
        
        if was_corrupt {
            warn!("Detected and fixed corrupted velocity");
        }
        
        // Additional game-specific limits
        velocity.linvel = velocity.linvel.clamp_length_max(config.physics.max_velocity);
        velocity.angvel = velocity.angvel.clamp_length_max(config.physics.max_angular_velocity);
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

/// Enhanced comprehensive physics safety system
/// Runs AFTER physics step to catch any corruption before other systems see it
pub fn apply_universal_physics_safeguards(
    mut query: Query<(Entity, &mut Velocity, &mut Transform), With<RigidBody>>,
    config: Res<GameConfig>,
) {
    for (entity, mut velocity, mut transform) in query.iter_mut() {
        // Use safe math utilities for comprehensive validation
        let velocity_corrupt = sanitize_velocity(&mut velocity);
        let transform_corrupt = sanitize_transform(&mut transform);
        
        if velocity_corrupt || transform_corrupt {
            warn!("Entity {:?} had corrupted physics data, fixed", entity);
        }
        
        // Apply game-specific bounds
        PhysicsUtilities::apply_world_bounds(&mut transform, &mut velocity, &config);
    }
}
