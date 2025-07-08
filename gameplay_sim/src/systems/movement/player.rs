//! ───────────────────────────────────────────────
//! System:   Human Player Movement System
//! Purpose:  Handles player character movement, animation, and behavior
//! Schedule: Update (continuous)
//! Reads:    Time, `ControlManager`, Velocity, Transform, `HumanMovement`, `HumanAnimation`, `HumanBehavior`
//! Writes:   Velocity, `HumanMovement`, `HumanAnimation`, `HumanBehavior`
//! Invariants:
//!   * Player movement respects physics constraints
//!   * Animation states sync with movement direction
//!   * Only active player entity can be controlled
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use std::cell::RefCell;
use game_core::prelude::*;
use crate::systems::input::{ControlManager, ControlAction};
thread_local! {
    static MOVEMENT_RNG: RefCell<rand::rngs::ThreadRng> = RefCell::new(rand::thread_rng());
}
pub fn human_player_movement(
    time: Res<Time>,
    control_manager: Res<ControlManager>,
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
    let Ok((mut velocity, transform, mut movement, mut animation, behavior)) = 
        player_query.single_mut() else {
        // Debug: Check if player exists but doesn't have ActiveEntity
        let all_players = player_query.iter().count();
        if all_players == 0 {
            warn!("No player entity found with ActiveEntity component!");
        }
        return;
    };
    let dt = time.delta_secs();
    // Velocity-based movement system (unified approach)
    let target_linear_velocity;
    let mut target_angular_velocity = Vec3::ZERO;
    
    let is_running = control_manager.is_control_active(ControlAction::Turbo);
    // Process input direction using UNIFIED control system
    let mut input_direction = Vec3::ZERO;
    if control_manager.is_control_active(ControlAction::Accelerate) {
        input_direction += *transform.forward();
    }
    if control_manager.is_control_active(ControlAction::Brake) {
        input_direction -= *transform.forward();
    }
    
    // Rotation using unified ControlManager
    let rotation_input = control_manager.get_control_value(ControlAction::Steer);
    
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
    // Calculate target linear velocity based on input (velocity-based system)
    if input_direction.length() > 0.0 {
        input_direction = input_direction.normalize();
        target_linear_velocity = input_direction * effective_max_speed;
        
        // Update movement tracking for animation
        movement.target_velocity = target_linear_velocity;
        movement.current_speed = target_linear_velocity.length();
    } else {
        target_linear_velocity = Vec3::ZERO;
        movement.target_velocity = Vec3::ZERO;
        movement.current_speed = 0.0;
    }
    
    // Calculate target angular velocity (velocity-based system)
    if rotation_input != 0.0 {
        target_angular_velocity.y = rotation_input * 1.8; // Human rotation speed
    }
    
    // Apply velocity directly (unified physics approach)
    velocity.linvel = target_linear_velocity;
    velocity.angvel = target_angular_velocity;
    
    // Update animation state only
    animation.is_walking = movement.current_speed > 0.3;
    animation.is_running = is_running && movement.current_speed > movement.max_speed * 1.0;
}

pub fn human_player_animation(
    player_query: Query<
        (&Transform, &HumanAnimation, &HumanMovement),
        With<Player>
    >,
    mut torso_query: Query<&mut Transform, (With<PlayerTorso>, Without<Player>)>,
    mut head_query: Query<&mut Transform, (With<PlayerHead>, Without<Player>, Without<PlayerTorso>)>,
    mut left_arm_query: Query<&mut Transform, (With<PlayerLeftArm>, Without<Player>, Without<PlayerTorso>, Without<PlayerHead>)>,
    mut right_arm_query: Query<&mut Transform, (With<PlayerRightArm>, Without<Player>, Without<PlayerTorso>, Without<PlayerHead>, Without<PlayerLeftArm>)>,
    mut left_leg_query: Query<&mut Transform, (With<PlayerLeftLeg>, Without<Player>, Without<PlayerTorso>, Without<PlayerHead>, Without<PlayerLeftArm>, Without<PlayerRightArm>)>,
    mut right_leg_query: Query<&mut Transform, (With<PlayerRightLeg>, Without<Player>, Without<PlayerTorso>, Without<PlayerHead>, Without<PlayerLeftArm>, Without<PlayerRightArm>, Without<PlayerLeftLeg>)>,
    time: Res<Time>,
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
            -walk_cycle_offset * 0.4 // Opposite phase from left leg
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
