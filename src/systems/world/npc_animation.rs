use crate::components::{
    HumanAnimation, HumanMovement, NPC, NPCHead, NPCLeftArm, NPCLeftLeg, NPCRightArm, NPCRightLeg,
    NPCTorso,
};
use bevy::prelude::*;

/// NPC animation system - mirrors player animation
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn npc_animation_system(
    time: Res<Time>,
    npc_query: Query<(Entity, &HumanAnimation, &HumanMovement), With<NPC>>,
    mut head_query: Query<
        (&ChildOf, &mut Transform),
        (
            With<NPCHead>,
            Without<NPCTorso>,
            Without<NPCLeftArm>,
            Without<NPCRightArm>,
            Without<NPCLeftLeg>,
            Without<NPCRightLeg>,
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
        ),
    >,
) {
    let time_elapsed = time.elapsed_secs();

    for (npc_entity, animation, movement) in npc_query.iter() {
        let speed = movement.current_speed;

        // Determine realistic step cadence (Hz) based on speed
        let cadence_hz = if animation.is_running {
            let t = ((speed - 3.0) / (8.0 - 3.0)).clamp(0.0, 1.0);
            2.6 + t * (3.2 - 2.6)
        } else {
            let t = ((speed - 0.5) / (2.0 - 0.5)).clamp(0.0, 1.0);
            1.6 + t * (2.2 - 1.6)
        };
        let step_omega = 2.0 * std::f32::consts::PI * cadence_hz;

        // Calculate animation values
        let walk_cycle = if animation.is_walking {
            (time_elapsed * step_omega).sin()
        } else {
            0.0
        };

        let walk_cycle_offset = if animation.is_walking {
            (time_elapsed * step_omega + std::f32::consts::PI).sin()
        } else {
            0.0
        };

        let breathing_cycle = (time_elapsed * animation.breathing_rate).sin();
        let idle_sway = (time_elapsed * 0.7).sin() * 0.5 + (time_elapsed * 1.1).cos() * 0.3;

        // Animate head
        for (child_of, mut head_transform) in head_query.iter_mut() {
            if child_of.0 != npc_entity {
                continue;
            }

            let head_bob = if animation.is_walking {
                walk_cycle * animation.head_bob_amplitude
            } else {
                breathing_cycle * 0.008
            };

            let head_sway = if animation.is_walking {
                walk_cycle * 0.5 * animation.body_sway_amplitude
            } else {
                idle_sway * 0.005
            };

            head_transform.translation.y = 1.2 + head_bob;
            head_transform.translation.x = head_sway;
            head_transform.rotation = Quat::IDENTITY;
        }

        // Animate torso
        for (child_of, mut torso_transform) in torso_query.iter_mut() {
            if child_of.0 != npc_entity {
                continue;
            }

            let body_sway = if animation.is_walking {
                walk_cycle * animation.body_sway_amplitude
            } else {
                idle_sway * 0.005
            };

            let body_breathing = breathing_cycle * 0.008;

            torso_transform.translation.y = 0.6 + body_breathing;
            torso_transform.translation.x = body_sway;

            if animation.is_running {
                torso_transform.rotation = Quat::from_rotation_x(-0.1);
            } else {
                torso_transform.rotation = Quat::IDENTITY;
            }
        }

        // Animate left arm
        for (child_of, mut left_arm_transform) in left_arm_query.iter_mut() {
            if child_of.0 != npc_entity {
                continue;
            }

            let arm_swing = if animation.is_walking {
                walk_cycle * 0.8
            } else {
                idle_sway * 0.05
            };

            left_arm_transform.translation.x = -0.4;
            left_arm_transform.translation.y = 0.7;
            left_arm_transform.translation.z = arm_swing * 0.35;
            left_arm_transform.rotation = Quat::from_rotation_x(arm_swing);
        }

        // Animate right arm
        for (child_of, mut right_arm_transform) in right_arm_query.iter_mut() {
            if child_of.0 != npc_entity {
                continue;
            }

            let arm_swing = if animation.is_walking {
                -walk_cycle * 0.8
            } else {
                -idle_sway * 0.05
            };

            right_arm_transform.translation.x = 0.4;
            right_arm_transform.translation.y = 0.7;
            right_arm_transform.translation.z = arm_swing * 0.35;
            right_arm_transform.rotation = Quat::from_rotation_x(arm_swing);
        }

        // Animate left leg
        for (child_of, mut left_leg_transform) in left_leg_query.iter_mut() {
            if child_of.0 != npc_entity {
                continue;
            }

            let leg_swing = if animation.is_walking {
                walk_cycle * 0.7
            } else {
                0.0
            };

            let leg_lift = if animation.is_walking {
                (walk_cycle * 0.5).max(0.0) * 0.15
            } else {
                0.0
            };

            left_leg_transform.translation.x = -0.15;
            left_leg_transform.translation.y = 0.0 + leg_lift;
            left_leg_transform.translation.z = leg_swing * 0.25;
            left_leg_transform.rotation = Quat::from_rotation_x(leg_swing);
        }

        // Animate right leg
        for (child_of, mut right_leg_transform) in right_leg_query.iter_mut() {
            if child_of.0 != npc_entity {
                continue;
            }

            let leg_swing = if animation.is_walking {
                walk_cycle_offset * 0.7
            } else {
                0.0
            };

            let leg_lift = if animation.is_walking {
                (walk_cycle_offset * 0.5).max(0.0) * 0.15
            } else {
                0.0
            };

            right_leg_transform.translation.x = 0.15;
            right_leg_transform.translation.y = 0.0 + leg_lift;
            right_leg_transform.translation.z = leg_swing * 0.25;
            right_leg_transform.rotation = Quat::from_rotation_x(leg_swing);
        }
    }
}
