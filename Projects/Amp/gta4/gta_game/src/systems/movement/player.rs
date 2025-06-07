use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use crate::components::{Player, ActiveEntity, HumanMovement, HumanAnimation, HumanBehavior};

pub fn human_player_movement(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<
        (
            &mut Velocity,
            &Transform,
            &mut HumanMovement,
            &mut HumanAnimation,
            &mut HumanBehavior,
        ),
        (With<Player>, With<ActiveEntity>),
    >,
) {
    let Ok((mut velocity, transform, mut movement, mut animation, mut behavior)) = 
        player_query.single_mut() else {
        // Debug: Check if player exists but doesn't have ActiveEntity
        let all_players = player_query.iter().count();
        if all_players == 0 {
            warn!("No player entity found with ActiveEntity component!");
        }
        return;
    };

    let dt = time.delta_secs();
    let mut rng = rand::thread_rng();

    // Update behavior timers
    behavior.input_delay_timer = (behavior.input_delay_timer - dt).max(0.0);

    // Process input with human reaction time
    let input_active = behavior.input_delay_timer <= 0.0;
    let mut input_direction = Vec3::ZERO;
    let mut rotation_input = 0.0;
    let is_running = input.pressed(KeyCode::ShiftLeft);

    if input_active {
        // Forward/backward movement
        if input.pressed(KeyCode::ArrowUp) {
            input_direction += *transform.forward();
        }
        if input.pressed(KeyCode::ArrowDown) {
            input_direction -= *transform.forward();
        }
        
        // Rotation
        if input.pressed(KeyCode::ArrowLeft) {
            rotation_input = 1.0;
        } else if input.pressed(KeyCode::ArrowRight) {
            rotation_input = -1.0;
        }

        // Add reaction delay for new inputs
        if input.any_just_pressed([KeyCode::ArrowUp, KeyCode::ArrowDown, 
                                  KeyCode::ArrowLeft, KeyCode::ArrowRight]) {
            behavior.input_delay_timer = behavior.reaction_time;
        }
    }

    // Handle stamina system
    let is_moving = input_direction.length() > 0.0;
    if is_moving && is_running {
        movement.stamina -= movement.stamina_drain_rate * dt;
        movement.stamina = movement.stamina.max(0.0);
    } else if !is_moving || !is_running {
        movement.stamina += movement.stamina_recovery_rate * dt;
        movement.stamina = movement.stamina.min(movement.max_stamina);
    }

    // Determine effective speed based on stamina and personality
    let stamina_factor = if movement.stamina < 20.0 {
        movement.tired_speed_modifier
    } else {
        1.0
    };

    let base_speed = if is_running && movement.stamina > 10.0 {
        movement.max_speed * 1.8
        } else {
        movement.max_speed
    };

    let effective_max_speed = base_speed * stamina_factor * 
                             behavior.personality_speed_modifier * 
                             behavior.movement_variation;

    // Natural acceleration/deceleration
    if input_direction.length() > 0.0 {
        input_direction = input_direction.normalize();
        movement.target_velocity = input_direction * effective_max_speed;
        
        // Add subtle directional drift for human imperfection
        if time.elapsed_secs() - behavior.last_direction_change > 0.5 {
            behavior.directional_drift = Vec3::new(
                rng.gen_range(-0.02..0.02),
                0.0,
                rng.gen_range(-0.02..0.02),
            );
            behavior.last_direction_change = time.elapsed_secs();
        }
        
        movement.target_velocity += behavior.directional_drift;
    } else {
        movement.target_velocity = Vec3::ZERO;
    }

    // Apply acceleration/deceleration with momentum
    let acceleration_rate = if movement.target_velocity.length() > movement.current_speed {
        movement.acceleration
    } else {
        movement.deceleration
    };

    movement.momentum = movement.momentum.lerp(movement.target_velocity, 
                                              acceleration_rate * dt);
    movement.current_speed = movement.momentum.length();

    // Apply natural variation to final velocity
    let velocity_variation = 1.0 + rng.gen_range(-0.05..0.05);
    let final_velocity = movement.momentum * velocity_variation;

    // Handle rotation with human imperfection
    let rotation_speed = 2.5 * behavior.confidence_level;
    let mut angular_velocity = rotation_input * rotation_speed;
    
    // Add subtle rotation drift when moving
    if is_moving {
        angular_velocity += rng.gen_range(-0.1..0.1);
    }

    // Update animation state
    animation.is_walking = movement.current_speed > 0.5;
    animation.is_running = is_running && movement.current_speed > movement.max_speed * 1.2;

    // Apply final velocities
    velocity.linvel = final_velocity;
    velocity.angvel.y = angular_velocity;
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
            idle_sway * 0.003
        };

        let body_breathing = breathing_cycle * 0.005;

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
