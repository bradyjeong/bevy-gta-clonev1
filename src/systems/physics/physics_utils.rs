use crate::config::GameConfig;
use crate::constants::{CHARACTER_GROUP, STATIC_GROUP, VEHICLE_GROUP};
use crate::util::safe_math::{Vec3SafeExt, validate_transform, validate_velocity};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Essential physics utilities for movement systems
#[derive(Default)]
pub struct PhysicsUtilities;

impl PhysicsUtilities {
    /// Validate and clamp velocity to safe ranges - single authority for game velocity limits
    pub fn clamp_velocity(velocity: &mut Velocity, config: &GameConfig) {
        // Use safe math for validation and fixing
        if validate_velocity(velocity) {
            warn!("Detected and fixed corrupted velocity");
        }

        // Apply game-specific limits using safe clamp extension
        velocity.linvel = Vec3SafeExt::clamp_length(velocity.linvel, config.physics.max_velocity);
        velocity.angvel =
            Vec3SafeExt::clamp_length(velocity.angvel, config.physics.max_angular_velocity);
    }

    /// Fixed delta-time for physics systems running in FixedUpdate
    /// Uses Bevy's fixed timestep for consistent physics behavior
    pub fn stable_dt(_time: &Time) -> f32 {
        // In Bevy 0.16, use fixed timestep for FixedUpdate systems
        1.0 / 60.0 // Standard 60Hz physics timestep
    }

    /// Emergency failsafe for extreme coordinate corruption (>100km)
    /// Only logs and disables entities - no more invisible walls
    pub fn emergency_coordinate_failsafe(
        transform: &mut Transform,
        entity: Entity,
        commands: &mut Commands,
    ) -> bool {
        const EMERGENCY_THRESHOLD: f32 = 100_000.0; // 100km - truly extreme

        let distance = transform.translation.length();
        if distance > EMERGENCY_THRESHOLD {
            error!(
                "Entity {:?} at extreme distance {:.1}km - disabling for safety",
                entity,
                distance / 1000.0
            );

            // Disable the entity instead of teleporting it
            commands.entity(entity).insert(RigidBodyDisabled);
            return true; // Indicates emergency action taken
        }

        false // No emergency action needed
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
        CollisionGroups::new(
            VEHICLE_GROUP,
            STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP,
        )
    }

    /// Get collision groups for characters (player, NPCs)
    pub fn character_groups() -> CollisionGroups {
        CollisionGroups::new(CHARACTER_GROUP, STATIC_GROUP | VEHICLE_GROUP)
    }
}

/// Enhanced comprehensive physics safety system
/// Runs AFTER physics step to catch any corruption before other systems see it
pub fn apply_universal_physics_safeguards(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Velocity, &mut Transform), With<RigidBody>>,
    config: Res<GameConfig>,
) {
    for (entity, mut velocity, mut transform) in query.iter_mut() {
        // Use safe math utilities for comprehensive validation
        let velocity_corrupt = validate_velocity(&mut velocity);
        let transform_corrupt = validate_transform(&mut transform);

        if velocity_corrupt || transform_corrupt {
            warn!("Entity {:?} had corrupted physics data, fixed", entity);
        }

        // Emergency failsafe only - no hard boundaries anymore
        PhysicsUtilities::emergency_coordinate_failsafe(&mut transform, entity, &mut commands);

        // Clamp velocity to prevent physics explosions (but allow free movement)
        PhysicsUtilities::clamp_velocity(&mut velocity, &config);
    }
}
