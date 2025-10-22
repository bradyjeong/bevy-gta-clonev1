use crate::config::GameConfig;

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

        // Apply game-specific limits using safe clamp extension from world_physics config
        velocity.linvel = Vec3SafeExt::clamp_length(
            velocity.linvel,
            config.world_physics.emergency_thresholds.max_velocity,
        );
        velocity.angvel = Vec3SafeExt::clamp_length(
            velocity.angvel,
            config
                .world_physics
                .emergency_thresholds
                .max_angular_velocity,
        );
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
        config: &GameConfig,
    ) -> bool {
        let emergency_threshold = config.world_physics.emergency_thresholds.max_coordinate;

        let distance = transform.translation.length();
        if distance > emergency_threshold {
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
    pub fn static_groups(config: &GameConfig) -> CollisionGroups {
        CollisionGroups::new(config.physics.static_group, Group::ALL)
    }

    /// Get collision groups for vehicles (cars, aircraft)
    pub fn vehicle_groups(config: &GameConfig) -> CollisionGroups {
        CollisionGroups::new(
            config.physics.vehicle_group,
            config.physics.static_group
                | config.physics.vehicle_group
                | config.physics.character_group,
        )
    }

    /// Get collision groups for characters (player, NPCs)
    pub fn character_groups(config: &GameConfig) -> CollisionGroups {
        CollisionGroups::new(
            config.physics.character_group,
            config.physics.static_group | config.physics.vehicle_group,
        )
    }
}

type PhysicsEntityQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static mut Velocity, &'static mut Transform),
    (With<RigidBody>, Without<RigidBodyDisabled>),
>;

/// Enhanced comprehensive physics safety system
/// Runs AFTER physics step to catch any corruption before other systems see it
pub fn apply_universal_physics_safeguards(
    mut commands: Commands,
    mut query: PhysicsEntityQuery,
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
        PhysicsUtilities::emergency_coordinate_failsafe(
            &mut transform,
            entity,
            &mut commands,
            &config,
        );

        // Clamp velocity to prevent physics explosions (but allow free movement)
        PhysicsUtilities::clamp_velocity(&mut velocity, &config);
    }
}
