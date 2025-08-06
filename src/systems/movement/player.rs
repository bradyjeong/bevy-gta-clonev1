use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::components::{Player, ActiveEntity, HumanMovement, HumanAnimation, HumanBehavior};
use crate::systems::input::{ControlManager, ControlAction};

#[derive(Resource)]
pub struct PlayerInputData {
    pub input_direction: Vec3,
    pub rotation_input: f32,
    pub is_running: bool,
}

impl Default for PlayerInputData {
    fn default() -> Self {
        Self {
            input_direction: Vec3::ZERO,
            rotation_input: 0.0,
            is_running: false,
        }
    }
}

pub fn read_input_system(
    control_manager: Res<ControlManager>,
    mut input_data: ResMut<PlayerInputData>,
    player_query: Query<&Transform, (With<Player>, With<ActiveEntity>)>,
) {
    let Ok(transform) = player_query.single() else {
        return;
    };

    input_data.input_direction = Vec3::ZERO;
    if control_manager.is_control_active(ControlAction::Accelerate) {
        input_data.input_direction += *transform.forward();
    }
    if control_manager.is_control_active(ControlAction::Brake) {
        input_data.input_direction -= *transform.forward();
    }
    
    input_data.rotation_input = control_manager.get_control_value(ControlAction::Steer);
    input_data.is_running = control_manager.is_control_active(ControlAction::Turbo);
}

pub fn stamina_system(
    time: Res<Time>,
    input_data: Res<PlayerInputData>,
    mut player_query: Query<&mut HumanMovement, (With<Player>, With<ActiveEntity>)>,
) {
    let Ok(mut movement) = player_query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    let is_moving = input_data.input_direction.length() > 0.0;
    
    if is_moving && input_data.is_running {
        movement.stamina -= movement.stamina_drain_rate * dt;
        movement.stamina = movement.stamina.max(0.0);
    } else if !is_moving || !input_data.is_running {
        movement.stamina += movement.stamina_recovery_rate * dt;
        movement.stamina = movement.stamina.min(movement.max_stamina);
    }
}

pub fn velocity_apply_system(
    input_data: Res<PlayerInputData>,
    mut player_query: Query<
        (&mut Velocity, &mut HumanMovement, &HumanBehavior),
        (With<Player>, With<ActiveEntity>),
    >,
) {
    let Ok((mut velocity, mut movement, behavior)) = player_query.single_mut() else {
        return;
    };

    let stamina_factor = if movement.stamina < 20.0 {
        movement.tired_speed_modifier
    } else {
        1.0
    };

    let base_speed = if input_data.is_running && movement.stamina > 10.0 {
        movement.max_speed * 1.8
    } else {
        movement.max_speed
    };

    let effective_max_speed = base_speed * stamina_factor * 
                             behavior.personality_speed_modifier * 
                             behavior.movement_variation;

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

    velocity.linvel = target_linear_velocity;
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
    animation.is_running = input_data.is_running && movement.current_speed > movement.max_speed * 1.0;
}

pub fn human_player_animation(
    time: Res<Time>,
    player_query: Query<
        (&Transform, &HumanAnimation, &HumanMovement),
        (With<Player>, With<ActiveEntity>),
    >,
    mut torso_query: Query<&mut Transform, (With<crate::components::PlayerTorso>, Without<Player>)>,
    mut head_query: Query<&mut Transform, (With<crate::components::PlayerHead>, Without<Player>, Without<crate::components::PlayerTorso>)>,
    mut left_arm_query: Query<&mut Transform, (With<crate::components::PlayerLeftArm>, Without<Player>, Without<crate::components::PlayerTorso>, Without<crate::components::PlayerHead>)>,
    mut right_arm_query: Query<&mut Transform, (With<crate::components::PlayerRightArm>, Without<Player>, Without<crate::components::PlayerTorso>, Without<crate::components::PlayerHead>, Without<crate::components::PlayerLeftArm>)>,
    mut left_leg_query: Query<&mut Transform, (With<crate::components::PlayerLeftLeg>, Without<Player>, Without<crate::components::PlayerTorso>, Without<crate::components::PlayerHead>, Without<crate::components::PlayerLeftArm>, Without<crate::components::PlayerRightArm>)>,
    mut right_leg_query: Query<&mut Transform, (With<crate::components::PlayerRightLeg>, Without<Player>, Without<crate::components::PlayerTorso>, Without<crate::components::PlayerHead>, Without<crate::components::PlayerLeftArm>, Without<crate::components::PlayerRightArm>, Without<crate::components::PlayerLeftLeg>)>,
) {
    let Ok((_player_transform, animation, _movement)) = player_query.single() else {
        return;
    };

    let _dt = time.delta_secs();
    let time_elapsed = time.elapsed_secs();

    // Calculate animation values
    let walk_cycle = if animation.is_walking {
        (time_elapsed * animation.step_frequency).sin()
    } else {
        0.0
    };

    let walk_cycle_offset = if animation.is_walking {
        (time_elapsed * animation.step_frequency + std::f32::consts::PI).sin()
    } else {
        0.0
    };

    let breathing_cycle = (time_elapsed * animation.breathing_rate).sin();
    let idle_sway = (time_elapsed * 0.7).sin() * 0.5 + (time_elapsed * 1.1).cos() * 0.3;

    // Apply head bobbing and breathing
    if let Ok(mut head_transform) = head_query.single_mut() {
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
    }

    // Apply torso swaying and breathing
    if let Ok(mut torso_transform) = torso_query.single_mut() {
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

    // Animate arms swinging
    if let Ok(mut left_arm_transform) = left_arm_query.single_mut() {
        let arm_swing = if animation.is_walking {
            walk_cycle * 0.5 // Arms swing opposite to legs
        } else {
            idle_sway * 0.05
        };

        left_arm_transform.translation.x = -0.4;
        left_arm_transform.translation.y = 0.7;
        left_arm_transform.translation.z = arm_swing * 0.2;
        left_arm_transform.rotation = Quat::from_rotation_x(arm_swing);
    }

    if let Ok(mut right_arm_transform) = right_arm_query.single_mut() {
        let arm_swing = if animation.is_walking {
            -walk_cycle * 0.5 // Opposite swing from left arm
        } else {
            -idle_sway * 0.05
        };

        right_arm_transform.translation.x = 0.4;
        right_arm_transform.translation.y = 0.7;
        right_arm_transform.translation.z = arm_swing * 0.2;
        right_arm_transform.rotation = Quat::from_rotation_x(arm_swing);
    }

    // Animate legs walking
    if let Ok(mut left_leg_transform) = left_leg_query.single_mut() {
        let leg_swing = if animation.is_walking {
            walk_cycle * 0.4
        } else {
            0.0
        };

        let leg_lift = if animation.is_walking {
            (walk_cycle * 0.5).max(0.0) * 0.1
        } else {
            0.0
        };

        left_leg_transform.translation.x = -0.15;
        left_leg_transform.translation.y = 0.0 + leg_lift;
        left_leg_transform.translation.z = leg_swing * 0.15;
        left_leg_transform.rotation = Quat::from_rotation_x(leg_swing);
    }

    if let Ok(mut right_leg_transform) = right_leg_query.single_mut() {
        let leg_swing = if animation.is_walking {
            walk_cycle_offset * 0.4 // Opposite phase from left leg
        } else {
            0.0
        };

        let leg_lift = if animation.is_walking {
            (walk_cycle_offset * 0.5).max(0.0) * 0.1
        } else {
            0.0
        };

        right_leg_transform.translation.x = 0.15;
        right_leg_transform.translation.y = 0.0 + leg_lift;
        right_leg_transform.translation.z = leg_swing * 0.15;
        right_leg_transform.rotation = Quat::from_rotation_x(leg_swing);
    }
}
