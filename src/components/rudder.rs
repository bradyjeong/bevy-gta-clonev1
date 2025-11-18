use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Rudder {
    pub max_angle: f32,
}
