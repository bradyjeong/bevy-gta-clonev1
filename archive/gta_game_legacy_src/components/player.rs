use bevy::prelude::*;
use rand::Rng;
use std::cell::RefCell;

thread_local! {
    static PLAYER_RNG: RefCell<rand::rngs::ThreadRng> = RefCell::new(rand::thread_rng());
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct ActiveEntity;

#[derive(Component)]
pub struct InCar(#[allow(dead_code)] pub Entity);

#[derive(Component)]
pub struct HumanMovement {
    pub acceleration: f32,
    pub deceleration: f32,
    pub max_speed: f32,
    pub current_speed: f32,
    pub target_velocity: Vec3,
    pub stamina: f32,
    pub max_stamina: f32,
    pub stamina_drain_rate: f32,
    pub stamina_recovery_rate: f32,
    pub tired_speed_modifier: f32,
}

impl Default for HumanMovement {
    fn default() -> Self {
        Self {
            acceleration: 32.0,
            deceleration: 50.0,
            max_speed: 4.5,
            current_speed: 0.0,
            target_velocity: Vec3::ZERO,
            stamina: 100.0,
            max_stamina: 100.0,
            stamina_drain_rate: 12.0,
            stamina_recovery_rate: 30.0,
            tired_speed_modifier: 0.7,
        }
    }
}

#[derive(Component)]
pub struct HumanAnimation {
    pub walk_cycle_time: f32,
    pub step_frequency: f32,
    pub head_bob_amplitude: f32,
    pub body_sway_amplitude: f32,
    pub breathing_rate: f32,
    pub idle_fidget_timer: f32,
    pub next_fidget_time: f32,
    pub is_walking: bool,
    pub is_running: bool,
}

impl Default for HumanAnimation {
    fn default() -> Self {
        Self {
            walk_cycle_time: 0.0,
            step_frequency: 3.0,
            head_bob_amplitude: 0.025,
            body_sway_amplitude: 0.015,
            breathing_rate: 1.4,
            idle_fidget_timer: 0.0,
            next_fidget_time: PLAYER_RNG.with(|rng| rng.borrow_mut().gen_range(3.0..8.0)),
            is_walking: false,
            is_running: false,
        }
    }
}

#[derive(Component)]
pub struct HumanBehavior {
    pub reaction_time: f32,
    pub input_delay_timer: f32,
    pub movement_variation: f32,
    pub directional_drift: Vec3,
    pub last_direction_change: f32,
    pub personality_speed_modifier: f32,
    pub confidence_level: f32,
}

impl Default for HumanBehavior {
    fn default() -> Self {
        Self {
            reaction_time: 0.0,
            input_delay_timer: 0.0,
            movement_variation: PLAYER_RNG.with(|rng| rng.borrow_mut().gen_range(0.95..1.05)),
            directional_drift: Vec3::ZERO,
            last_direction_change: 0.0,
            personality_speed_modifier: PLAYER_RNG.with(|rng| rng.borrow_mut().gen_range(0.95..1.05)),
            confidence_level: PLAYER_RNG.with(|rng| rng.borrow_mut().gen_range(0.8..1.0)),
        }
    }
}

#[derive(Component)]
pub struct PlayerBody {
    pub base_transform: Transform,
    pub head_offset: Vec3,
    pub body_offset: Vec3,
}

impl Default for PlayerBody {
    fn default() -> Self {
        Self {
            base_transform: Transform::IDENTITY,
            head_offset: Vec3::new(0.0, 1.2, 0.0),
            body_offset: Vec3::new(0.0, 0.6, 0.0),
        }
    }
}

#[derive(Component)]
pub struct PlayerHead;

#[derive(Component)]
pub struct PlayerBodyMesh;

#[derive(Component)]
pub struct PlayerTorso;

#[derive(Component)]
pub struct PlayerLeftArm;

#[derive(Component)]
pub struct PlayerRightArm;

#[derive(Component)]
pub struct PlayerLeftLeg;

#[derive(Component)]
pub struct PlayerRightLeg;

#[derive(Component)]
pub struct BodyPart {
    pub rest_position: Vec3,
    pub rest_rotation: Quat,
    pub animation_offset: Vec3,
    pub animation_rotation: Quat,
}
