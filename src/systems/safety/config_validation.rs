use crate::components::world::WorldBounds;
use crate::config::GameConfig;
use bevy::prelude::*;

/// Startup validation to prevent physics explosions
pub fn validate_physics_config(
    config: Res<GameConfig>,
    bounds: Res<WorldBounds>,
    f16_query: Query<&crate::components::SimpleF16Specs>,
    helicopter_query: Query<&crate::components::SimpleHelicopterSpecs>,
    car_query: Query<&crate::components::SimpleCarSpecs>,
) {
    // Critical: max_velocity must be reasonable to prevent coordinate explosion
    let max_vel = config.physics.max_velocity;
    let world_size = (bounds.max_x - bounds.min_x).max(bounds.max_z - bounds.min_z);

    if max_vel > world_size / 5.0 {
        panic!(
            "Physics config error: max_velocity ({}) exceeds one-fifth of world bounds ({}). This can cause coordinate explosion.",
            max_vel,
            world_size / 5.0
        );
    }

    if max_vel > 1500.0 {
        warn!(
            "Physics config: max_velocity ({}) is very high (>Mach 4.5). Consider reducing to prevent instability.",
            max_vel
        );
    }

    // Check for reasonable angular velocity limits
    let max_ang_vel = config.physics.max_angular_velocity;
    if max_ang_vel > 100.0 {
        warn!(
            "Physics config: max_angular_velocity ({}) is very high. Consider reducing to prevent rotation chaos.",
            max_ang_vel
        );
    }

    // Validate vehicle specs against physics limits to prevent Rapier panics
    for specs in f16_query.iter() {
        let max_f16_vel = specs.max_forward_speed * specs.afterburner_multiplier;
        if max_f16_vel > max_vel {
            panic!(
                "F16 config error: max_forward_speed * afterburner_multiplier ({max_f16_vel:.1}) exceeds max_velocity ({max_vel}). Reduce speed limits.",
            );
        }

        if specs.roll_rate_max > max_ang_vel
            || specs.pitch_rate_max > max_ang_vel
            || specs.yaw_rate_max > max_ang_vel
        {
            panic!("F16 config error: rotation rates exceed max_angular_velocity ({max_ang_vel})",);
        }
    }

    for specs in helicopter_query.iter() {
        let max_heli_vel = specs
            .forward_speed
            .max(specs.lateral_speed)
            .max(specs.vertical_speed);
        if max_heli_vel > max_vel {
            panic!(
                "Helicopter config error: movement speeds ({max_heli_vel:.1}) exceed max_velocity ({max_vel})",
            );
        }

        if specs.yaw_rate > max_ang_vel
            || specs.pitch_rate > max_ang_vel
            || specs.roll_rate > max_ang_vel
        {
            panic!(
                "Helicopter config error: rotation rates exceed max_angular_velocity ({max_ang_vel})",
            );
        }
    }

    for specs in car_query.iter() {
        if specs.base_speed > max_vel {
            panic!(
                "Car config error: base_speed ({}) exceeds max_velocity ({})",
                specs.base_speed, max_vel
            );
        }

        if specs.rotation_speed > max_ang_vel {
            panic!(
                "Car config error: rotation_speed ({}) exceeds max_angular_velocity ({})",
                specs.rotation_speed, max_ang_vel
            );
        }
    }

    let world_size = (bounds.max_x - bounds.min_x).max(bounds.max_z - bounds.min_z);
    info!(
        "Physics config validated: max_velocity={}, max_angular_velocity={}, world_size={:.1}",
        max_vel, max_ang_vel, world_size
    );
}
