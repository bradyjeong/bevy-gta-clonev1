use bevy::prelude::*;

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
}

impl Default for HumanMovement {
    fn default() -> Self {
        Self {
            acceleration: 32.0,
            deceleration: 50.0,
            max_speed: 4.5, // Realistic walking speed (4.5 m/s = 16 km/h)
            current_speed: 0.0,
            target_velocity: Vec3::ZERO,
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
    pub is_swimming: bool,
    // Swimming animation fields
    pub swim_stroke_cycle: f32,
    pub swim_stroke_frequency: f32,
    pub swim_kick_cycle: f32,
    pub swim_speed: f32,
}

impl Default for HumanAnimation {
    fn default() -> Self {
        Self {
            walk_cycle_time: 0.0,
            step_frequency: 2.0 * std::f32::consts::PI * 1.9, // ~1.9 Hz walking cadence
            head_bob_amplitude: 0.025,
            body_sway_amplitude: 0.015,
            breathing_rate: 1.4,
            idle_fidget_timer: 0.0,
            next_fidget_time: 5.0, // Default 5.0 seconds - randomization can be done at spawn time
            is_walking: false,
            is_running: false,
            is_swimming: false,
            swim_stroke_cycle: 0.0,
            swim_stroke_frequency: 2.0 * std::f32::consts::PI * 0.8, // ~0.8 Hz stroke rate
            swim_kick_cycle: 0.0,
            swim_speed: 0.0,
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
pub struct PlayerLeftFoot;

#[derive(Component)]
pub struct PlayerRightFoot;

#[derive(Component)]
pub struct BodyPart {
    pub rest_position: Vec3,
    pub rest_rotation: Quat,
    pub animation_offset: Vec3,
    pub animation_rotation: Quat,
}
