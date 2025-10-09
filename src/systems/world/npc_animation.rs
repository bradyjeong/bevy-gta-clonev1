use crate::components::{
    HumanAnimation, HumanMovement, NPC, NPCHead, NPCLeftArm, NPCLeftFoot, NPCLeftLeg, NPCRightArm,
    NPCRightFoot, NPCRightLeg, NPCTorso,
};
use bevy::prelude::*;
use std::collections::HashMap;

struct AnimationValues {
    walk_cycle: f32,
    walk_cycle_offset: f32,
    breathing_cycle: f32,
    idle_sway: f32,
    is_walking: bool,
    is_running: bool,
    head_bob_amplitude: f32,
    body_sway_amplitude: f32,
}

impl AnimationValues {
    fn calculate(time: f32, animation: &HumanAnimation, movement: &HumanMovement) -> Self {
        let speed = movement.current_speed;

        // Ensure is_running implies is_walking for consistency
        let is_walking = animation.is_walking || animation.is_running;
        let is_running = animation.is_running && is_walking;

        let cadence_hz = if is_running {
            let t = ((speed - 3.0) / (8.0 - 3.0)).clamp(0.0, 1.0);
            2.6 + t * (3.2 - 2.6)
        } else {
            let t = ((speed - 0.5) / (2.0 - 0.5)).clamp(0.0, 1.0);
            1.6 + t * (2.2 - 1.6)
        };
        let step_omega = 2.0 * std::f32::consts::PI * cadence_hz;

        let walk_cycle = if is_walking {
            (time * step_omega).sin()
        } else {
            0.0
        };

        let walk_cycle_offset = if is_walking {
            (time * step_omega + std::f32::consts::PI).sin()
        } else {
            0.0
        };

        let breathing_cycle = (time * animation.breathing_rate).sin();
        let idle_sway = (time * 0.7).sin() * 0.5 + (time * 1.1).cos() * 0.3;

        Self {
            walk_cycle,
            walk_cycle_offset,
            breathing_cycle,
            idle_sway,
            is_walking,
            is_running,
            head_bob_amplitude: animation.head_bob_amplitude,
            body_sway_amplitude: animation.body_sway_amplitude,
        }
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn npc_animation_system(
    time: Res<Time>,
    npc_data: Query<(Entity, &HumanAnimation, &HumanMovement), With<NPC>>,
    mut head_query: Query<
        (&ChildOf, &mut Transform),
        (
            With<NPCHead>,
            Without<NPCTorso>,
            Without<NPCLeftArm>,
            Without<NPCRightArm>,
            Without<NPCLeftLeg>,
            Without<NPCRightLeg>,
            Without<NPCLeftFoot>,
            Without<NPCRightFoot>,
        ),
    >,
    mut torso_query: Query<
        (&ChildOf, &mut Transform),
        (
            With<NPCTorso>,
            Without<NPCHead>,
            Without<NPCLeftArm>,
            Without<NPCRightArm>,
            Without<NPCLeftLeg>,
            Without<NPCRightLeg>,
            Without<NPCLeftFoot>,
            Without<NPCRightFoot>,
        ),
    >,
    mut left_arm_query: Query<
        (&ChildOf, &mut Transform),
        (
            With<NPCLeftArm>,
            Without<NPCHead>,
            Without<NPCTorso>,
            Without<NPCRightArm>,
            Without<NPCLeftLeg>,
            Without<NPCRightLeg>,
            Without<NPCLeftFoot>,
            Without<NPCRightFoot>,
        ),
    >,
    mut right_arm_query: Query<
        (&ChildOf, &mut Transform),
        (
            With<NPCRightArm>,
            Without<NPCHead>,
            Without<NPCTorso>,
            Without<NPCLeftArm>,
            Without<NPCLeftLeg>,
            Without<NPCRightLeg>,
            Without<NPCLeftFoot>,
            Without<NPCRightFoot>,
        ),
    >,
    mut left_leg_query: Query<
        (&ChildOf, &mut Transform),
        (
            With<NPCLeftLeg>,
            Without<NPCHead>,
            Without<NPCTorso>,
            Without<NPCLeftArm>,
            Without<NPCRightArm>,
            Without<NPCRightLeg>,
            Without<NPCLeftFoot>,
            Without<NPCRightFoot>,
        ),
    >,
    mut right_leg_query: Query<
        (&ChildOf, &mut Transform),
        (
            With<NPCRightLeg>,
            Without<NPCHead>,
            Without<NPCTorso>,
            Without<NPCLeftArm>,
            Without<NPCRightArm>,
            Without<NPCLeftLeg>,
            Without<NPCLeftFoot>,
            Without<NPCRightFoot>,
        ),
    >,
    mut left_foot_query: Query<
        (&ChildOf, &mut Transform),
        (
            With<NPCLeftFoot>,
            Without<NPCHead>,
            Without<NPCTorso>,
            Without<NPCLeftArm>,
            Without<NPCRightArm>,
            Without<NPCLeftLeg>,
            Without<NPCRightLeg>,
            Without<NPCRightFoot>,
        ),
    >,
    mut right_foot_query: Query<
        (&ChildOf, &mut Transform),
        (
            With<NPCRightFoot>,
            Without<NPCHead>,
            Without<NPCTorso>,
            Without<NPCLeftArm>,
            Without<NPCRightArm>,
            Without<NPCLeftLeg>,
            Without<NPCRightLeg>,
            Without<NPCLeftFoot>,
        ),
    >,
) {
    let time_elapsed = time.elapsed_secs();

    // Precompute animation values once per NPC (instead of 8× per frame)
    let mut anim_cache = HashMap::new();
    for (entity, animation, movement) in npc_data.iter() {
        anim_cache.insert(
            entity,
            AnimationValues::calculate(time_elapsed, animation, movement),
        );
    }

    for (child_of, mut head_transform) in head_query.iter_mut() {
        if let Some(anim) = anim_cache.get(&child_of.0) {
            let head_bob = if anim.is_walking {
                anim.walk_cycle * anim.head_bob_amplitude
            } else {
                anim.breathing_cycle * 0.008
            };

            let head_sway = if anim.is_walking {
                anim.walk_cycle * 0.5 * anim.body_sway_amplitude
            } else {
                anim.idle_sway * 0.005
            };

            head_transform.translation.y = 1.2 + head_bob;
            head_transform.translation.x = head_sway;
            head_transform.rotation = Quat::IDENTITY;
        }
    }

    for (child_of, mut torso_transform) in torso_query.iter_mut() {
        if let Some(anim) = anim_cache.get(&child_of.0) {
            let body_sway = if anim.is_walking {
                anim.walk_cycle * anim.body_sway_amplitude
            } else {
                anim.idle_sway * 0.005
            };

            let body_breathing = anim.breathing_cycle * 0.008;

            torso_transform.translation.y = 0.6 + body_breathing;
            torso_transform.translation.x = body_sway;

            if anim.is_running {
                torso_transform.rotation = Quat::from_rotation_x(-0.1);
            } else {
                torso_transform.rotation = Quat::IDENTITY;
            }
        }
    }

    for (child_of, mut left_arm_transform) in left_arm_query.iter_mut() {
        if let Some(anim) = anim_cache.get(&child_of.0) {
            let arm_swing = if anim.is_walking {
                anim.walk_cycle * 0.8
            } else {
                anim.idle_sway * 0.05
            };

            left_arm_transform.translation.x = -0.4;
            left_arm_transform.translation.y = 0.7;
            left_arm_transform.translation.z = arm_swing * 0.35;
            left_arm_transform.rotation = Quat::from_rotation_x(arm_swing);
        }
    }

    for (child_of, mut right_arm_transform) in right_arm_query.iter_mut() {
        if let Some(anim) = anim_cache.get(&child_of.0) {
            let arm_swing = if anim.is_walking {
                -anim.walk_cycle * 0.8
            } else {
                -anim.idle_sway * 0.05
            };

            right_arm_transform.translation.x = 0.4;
            right_arm_transform.translation.y = 0.7;
            right_arm_transform.translation.z = arm_swing * 0.35;
            right_arm_transform.rotation = Quat::from_rotation_x(arm_swing);
        }
    }

    for (child_of, mut left_leg_transform) in left_leg_query.iter_mut() {
        if let Some(anim) = anim_cache.get(&child_of.0) {
            let leg_swing = if anim.is_walking {
                anim.walk_cycle * 0.7
            } else {
                0.0
            };

            let leg_lift = if anim.is_walking {
                (anim.walk_cycle * 0.5).max(0.0) * 0.15
            } else {
                0.0
            };

            left_leg_transform.translation.x = -0.15;
            left_leg_transform.translation.y = 0.0 + leg_lift;
            left_leg_transform.translation.z = leg_swing * 0.25;
            left_leg_transform.rotation = Quat::from_rotation_x(leg_swing);
        }
    }

    for (child_of, mut right_leg_transform) in right_leg_query.iter_mut() {
        if let Some(anim) = anim_cache.get(&child_of.0) {
            let leg_swing = if anim.is_walking {
                anim.walk_cycle_offset * 0.7
            } else {
                0.0
            };

            let leg_lift = if anim.is_walking {
                (anim.walk_cycle_offset * 0.5).max(0.0) * 0.15
            } else {
                0.0
            };

            right_leg_transform.translation.x = 0.15;
            right_leg_transform.translation.y = 0.0 + leg_lift;
            right_leg_transform.translation.z = leg_swing * 0.25;
            right_leg_transform.rotation = Quat::from_rotation_x(leg_swing);
        }
    }

    for (child_of, mut left_foot_transform) in left_foot_query.iter_mut() {
        if let Some(anim) = anim_cache.get(&child_of.0) {
            let leg_swing = if anim.is_walking {
                anim.walk_cycle * 0.7
            } else {
                0.0
            };

            let leg_lift = if anim.is_walking {
                (anim.walk_cycle * 0.5).max(0.0) * 0.15
            } else {
                0.0
            };

            left_foot_transform.translation.x = -0.15;
            left_foot_transform.translation.y = -0.4 + leg_lift;
            left_foot_transform.translation.z = leg_swing * 0.25;
            left_foot_transform.rotation = Quat::from_rotation_x(leg_swing * 0.5);
        }
    }

    for (child_of, mut right_foot_transform) in right_foot_query.iter_mut() {
        if let Some(anim) = anim_cache.get(&child_of.0) {
            let leg_swing = if anim.is_walking {
                anim.walk_cycle_offset * 0.7
            } else {
                0.0
            };

            let leg_lift = if anim.is_walking {
                (anim.walk_cycle_offset * 0.5).max(0.0) * 0.15
            } else {
                0.0
            };

            right_foot_transform.translation.x = 0.15;
            right_foot_transform.translation.y = -0.4 + leg_lift;
            right_foot_transform.translation.z = leg_swing * 0.25;
            right_foot_transform.rotation = Quat::from_rotation_x(leg_swing * 0.5);
        }
    }
}
