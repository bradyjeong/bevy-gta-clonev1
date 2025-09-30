#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::components::ControlState;
use crate::components::player::{PlayerLeftFoot, PlayerRightFoot};
use crate::components::{ActiveEntity, HumanAnimation, HumanMovement, Player};

#[derive(Resource)]
pub struct PlayerInputData {
    pub input_direction: Vec3,
    pub rotation_input: f32,
    pub vertical_input: f32, // NEW: W = +1, S = -1
    pub is_running: bool,
}

impl Default for PlayerInputData {
    fn default() -> Self {
        Self {
            input_direction: Vec3::ZERO,
            rotation_input: 0.0,
            vertical_input: 0.0, // NEW
            is_running: false,
        }
    }
}

pub fn read_input_system(
    mut input_data: ResMut<PlayerInputData>,
    player_query: Query<(&Transform, &ControlState), (With<Player>, With<ActiveEntity>)>,
) {
    let Ok((transform, control_state)) = player_query.single() else {
        return;
    };

    input_data.input_direction = Vec3::ZERO;
    if control_state.is_accelerating() {
        input_data.input_direction += *transform.forward();
    }
    if control_state.is_braking() {
        input_data.input_direction -= *transform.forward();
    }

    input_data.rotation_input = control_state.steering;
    input_data.is_running = control_state.run;
    input_data.vertical_input = control_state.vertical; // NEW: Consume W/S for diving
}

pub fn velocity_apply_system(
    input_data: Res<PlayerInputData>,
    mut player_query: Query<
        (&mut Velocity, &mut HumanMovement),
        (
            With<Player>,
            With<ActiveEntity>,
            Without<crate::systems::swimming::Swimming>,
        ), // NEW FILTER
    >,
) {
    let Ok((mut velocity, mut movement)) = player_query.single_mut() else {
        return;
    };

    let effective_max_speed = if input_data.is_running {
        movement.max_speed * 1.8 // Sprint speed (8 m/s realistic)
    } else {
        movement.max_speed // Walk speed
    };

    let target_linear_velocity = if input_data.input_direction.length() > 0.0 {
        let normalized_input = input_data.input_direction.normalize();
        normalized_input * effective_max_speed
    } else {
        Vec3::ZERO
    };

    let target_angular_velocity = if input_data.rotation_input != 0.0 {
        Vec3::new(0.0, input_data.rotation_input * 1.8, 0.0)
    } else {
        Vec3::ZERO
    };

    movement.target_velocity = target_linear_velocity;
    movement.current_speed = target_linear_velocity.length();

    // Preserve gravity in Y-axis like NPCs do
    velocity.linvel = Vec3::new(
        target_linear_velocity.x,
        velocity.linvel.y, // Preserve gravity/falling
        target_linear_velocity.z,
    );
    velocity.angvel = target_angular_velocity;
}

pub fn animation_flag_system(
    input_data: Res<PlayerInputData>,
    mut player_query: Query<
        (&mut HumanAnimation, &HumanMovement),
        (With<Player>, With<ActiveEntity>),
    >,
) {
    let Ok((mut animation, movement)) = player_query.single_mut() else {
        return;
    };

    animation.is_walking = movement.current_speed > 0.3;
    animation.is_running = input_data.is_running;
}

pub fn human_player_animation(
    time: Res<Time>,
    player_query: Query<
        (&Transform, &HumanAnimation, &HumanMovement),
        (With<Player>, With<ActiveEntity>),
    >,
    mut torso_query: Query<&mut Transform, (With<crate::components::PlayerTorso>, Without<Player>)>,
    mut head_query: Query<
        &mut Transform,
        (
            With<crate::components::PlayerHead>,
            Without<Player>,
            Without<crate::components::PlayerTorso>,
        ),
    >,
    mut left_arm_query: Query<
        &mut Transform,
        (
            With<crate::components::PlayerLeftArm>,
            Without<Player>,
            Without<crate::components::PlayerTorso>,
            Without<crate::components::PlayerHead>,
        ),
    >,
    mut right_arm_query: Query<
        &mut Transform,
        (
            With<crate::components::PlayerRightArm>,
            Without<Player>,
            Without<crate::components::PlayerTorso>,
            Without<crate::components::PlayerHead>,
            Without<crate::components::PlayerLeftArm>,
        ),
    >,
    mut left_leg_query: Query<
        &mut Transform,
        (
            With<crate::components::PlayerLeftLeg>,
            Without<Player>,
            Without<crate::components::PlayerTorso>,
            Without<crate::components::PlayerHead>,
            Without<crate::components::PlayerLeftArm>,
            Without<crate::components::PlayerRightArm>,
        ),
    >,
    mut right_leg_query: Query<
        &mut Transform,
        (
            With<crate::components::PlayerRightLeg>,
            Without<Player>,
            Without<crate::components::PlayerTorso>,
            Without<crate::components::PlayerHead>,
            Without<crate::components::PlayerLeftArm>,
            Without<crate::components::PlayerRightArm>,
            Without<crate::components::PlayerLeftLeg>,
        ),
    >,
    mut left_foot_query: Query<
        &mut Transform,
        (
            With<PlayerLeftFoot>,
            Without<Player>,
            Without<crate::components::PlayerTorso>,
            Without<crate::components::PlayerHead>,
            Without<crate::components::PlayerLeftArm>,
            Without<crate::components::PlayerRightArm>,
            Without<crate::components::PlayerLeftLeg>,
            Without<crate::components::PlayerRightLeg>,
        ),
    >,
    mut right_foot_query: Query<
        &mut Transform,
        (
            With<PlayerRightFoot>,
            Without<Player>,
            Without<crate::components::PlayerTorso>,
            Without<crate::components::PlayerHead>,
            Without<crate::components::PlayerLeftArm>,
            Without<crate::components::PlayerRightArm>,
            Without<crate::components::PlayerLeftLeg>,
            Without<crate::components::PlayerRightLeg>,
            Without<PlayerLeftFoot>,
        ),
    >,
) {
    let Ok((_player_transform, animation, movement)) = player_query.single() else {
        return;
    };

    // Debug animation system running
    if (time.elapsed_secs() % 3.0) < 0.016 {
        info!(
            "🎭 ANIMATION DEBUG: is_swimming={}, is_walking={}, speed={:.2}",
            animation.is_swimming, animation.is_walking, movement.current_speed
        );
    }

    let _dt = time.delta_secs();
    let time_elapsed = time.elapsed_secs();

    // Determine realistic step cadence (Hz) based on speed and running state
    let speed = movement.current_speed;
    let cadence_hz = if animation.is_running {
        // Running: ~2.6–3.2 Hz across 3–8 m/s
        let t = ((speed - 3.0) / (8.0 - 3.0)).clamp(0.0, 1.0);
        2.6 + t * (3.2 - 2.6)
    } else {
        // Walking: ~1.6–2.2 Hz across 0.5–2.0 m/s
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

    // Apply head animation - walking, swimming, or idle
    if let Ok(mut head_transform) = head_query.single_mut() {
        if animation.is_swimming {
            // Swimming: Head turns side to side for breathing
            let breath_phase = (time_elapsed * animation.swim_stroke_frequency * 0.5).sin();
            let head_turn = breath_phase * 0.3; // Side-to-side breathing motion
            let head_bob = breath_phase * 0.05; // Slight vertical bob

            head_transform.translation.y = 1.2 + head_bob;
            head_transform.translation.x = head_turn * 0.1;
            head_transform.rotation = Quat::from_rotation_y(head_turn);
        } else {
            // Walking/idle: normal head movement
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
    }

    // Apply torso animation - walking, swimming, or idle
    if let Ok(mut torso_transform) = torso_query.single_mut() {
        if animation.is_swimming {
            // Swimming: Body roll with strokes + streamlined position
            let stroke_phase = (time_elapsed * animation.swim_stroke_frequency).sin();
            let body_roll = stroke_phase * 0.2; // Roll left/right with arm strokes
            let streamline_lean = -0.15; // Slight forward lean for streamlined position

            torso_transform.translation.y = 0.6;
            torso_transform.translation.x = body_roll * 0.1;
            torso_transform.rotation =
                Quat::from_rotation_x(streamline_lean) * Quat::from_rotation_z(body_roll);
        } else {
            // Walking/idle: normal torso movement
            let body_sway = if animation.is_walking {
                walk_cycle * animation.body_sway_amplitude
            } else {
                idle_sway * 0.005
            };

            let body_breathing = breathing_cycle * 0.008;

            torso_transform.translation.y = 0.6 + body_breathing;
            torso_transform.translation.x = body_sway;

            // Slight forward lean when running
            if animation.is_running {
                torso_transform.rotation = Quat::from_rotation_x(-0.1);
            } else {
                torso_transform.rotation = Quat::IDENTITY;
            }
        }
    }

    // Animate arms - walking or swimming
    if let Ok(mut left_arm_transform) = left_arm_query.single_mut() {
        if animation.is_swimming {
            // Debug - confirm we're in swimming branch
            if (time_elapsed % 3.0) < 0.016 {
                info!("🔥 ENTERING SWIMMING ARM ANIMATION BRANCH");
            }
            // Swimming: Freestyle stroke - left arm leads
            let stroke_phase = (time_elapsed * animation.swim_stroke_frequency).sin();
            let arm_forward = stroke_phase * 1.2; // Forward/back stroke motion
            let arm_down = (stroke_phase * 0.5).max(0.0) * 0.3; // Downward catch phase

            // Debug animation values occasionally
            if (time_elapsed % 2.0) < 0.016 {
                info!(
                    "🏊 SWIM ANIM DEBUG: stroke_freq={:.2}, stroke_phase={:.2}, arm_forward={:.2}",
                    animation.swim_stroke_frequency, stroke_phase, arm_forward
                );
            }

            left_arm_transform.translation.x = -0.4;
            left_arm_transform.translation.y = 0.7 - arm_down;
            left_arm_transform.translation.z = arm_forward * 0.6;
            left_arm_transform.rotation = Quat::from_rotation_x(arm_forward * 1.2);
        } else {
            // Walking/idle: arm swing or reset from swimming
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
    }

    if let Ok(mut right_arm_transform) = right_arm_query.single_mut() {
        if animation.is_swimming {
            // Swimming: Freestyle stroke - right arm offset by π (alternating)
            let stroke_phase =
                (time_elapsed * animation.swim_stroke_frequency + std::f32::consts::PI).sin();
            let arm_forward = stroke_phase * 1.2;
            let arm_down = (stroke_phase * 0.5).max(0.0) * 0.3;

            right_arm_transform.translation.x = 0.4;
            right_arm_transform.translation.y = 0.7 - arm_down;
            right_arm_transform.translation.z = arm_forward * 0.6;
            right_arm_transform.rotation = Quat::from_rotation_x(arm_forward * 1.2);
        } else {
            // Walking: arm swing
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
    }

    // Animate legs - walking or swimming
    if let Ok(mut left_leg_transform) = left_leg_query.single_mut() {
        if animation.is_swimming {
            // Swimming: Flutter kick - rapid up/down motion
            let kick_phase = (time_elapsed * animation.swim_stroke_frequency * 3.0).sin(); // 3x faster than arms
            let leg_kick = kick_phase * 0.4; // Up/down kicking motion
            let leg_bend = (kick_phase * 0.5).abs() * 0.8; // Knee bending

            left_leg_transform.translation.x = -0.15;
            left_leg_transform.translation.y = 0.0 + leg_kick * 0.2;
            left_leg_transform.translation.z = -leg_bend * 0.3; // Bend backward
            left_leg_transform.rotation = Quat::from_rotation_x(-leg_bend);
        } else {
            // Walking: leg swing
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
    }

    if let Ok(mut right_leg_transform) = right_leg_query.single_mut() {
        if animation.is_swimming {
            // Swimming: Flutter kick - alternating with left leg
            let kick_phase = (time_elapsed * animation.swim_stroke_frequency * 3.0
                + std::f32::consts::PI * 0.5)
                .sin();
            let leg_kick = kick_phase * 0.4;
            let leg_bend = (kick_phase * 0.5).abs() * 0.8;

            right_leg_transform.translation.x = 0.15;
            right_leg_transform.translation.y = 0.0 + leg_kick * 0.2;
            right_leg_transform.translation.z = -leg_bend * 0.3;
            right_leg_transform.rotation = Quat::from_rotation_x(-leg_bend);
        } else {
            // Walking: leg swing
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

    // Animate feet to follow their respective legs
    if let Ok(mut left_foot_transform) = left_foot_query.single_mut() {
        if animation.is_swimming {
            // Swimming: feet follow leg kick motion
            let kick_phase = (time_elapsed * animation.swim_stroke_frequency * 3.0).sin();
            let leg_kick = kick_phase * 0.4;
            let leg_bend = (kick_phase * 0.5).abs() * 0.8;

            left_foot_transform.translation.x = -0.15;
            left_foot_transform.translation.y = -0.4 + leg_kick * 0.2; // Follow leg Y movement
            left_foot_transform.translation.z = -leg_bend * 0.3; // Match leg bending exactly
            left_foot_transform.rotation = Quat::from_rotation_x(-leg_bend * 0.5); // Partial leg rotation
        } else {
            // Walking: feet follow leg swing and lifting
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

            left_foot_transform.translation.x = -0.15;
            left_foot_transform.translation.y = -0.4 + leg_lift; // Follow leg lifting
            left_foot_transform.translation.z = leg_swing * 0.25; // Match leg swing exactly
            left_foot_transform.rotation = Quat::from_rotation_x(leg_swing * 0.5); // Partial leg rotation
        }
    }

    if let Ok(mut right_foot_transform) = right_foot_query.single_mut() {
        if animation.is_swimming {
            // Swimming: feet follow leg kick motion (alternating)
            let kick_phase = (time_elapsed * animation.swim_stroke_frequency * 3.0
                + std::f32::consts::PI * 0.5)
                .sin();
            let leg_kick = kick_phase * 0.4;
            let leg_bend = (kick_phase * 0.5).abs() * 0.8;

            right_foot_transform.translation.x = 0.15;
            right_foot_transform.translation.y = -0.4 + leg_kick * 0.2; // Follow leg Y movement
            right_foot_transform.translation.z = -leg_bend * 0.3; // Match leg bending exactly
            right_foot_transform.rotation = Quat::from_rotation_x(-leg_bend * 0.5); // Partial leg rotation
        } else {
            // Walking: feet follow leg swing and lifting
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

            right_foot_transform.translation.x = 0.15;
            right_foot_transform.translation.y = -0.4 + leg_lift; // Follow leg lifting
            right_foot_transform.translation.z = leg_swing * 0.25; // Match leg swing exactly
            right_foot_transform.rotation = Quat::from_rotation_x(leg_swing * 0.5); // Partial leg rotation
        }
    }
}
