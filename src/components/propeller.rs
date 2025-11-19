use bevy::prelude::*;

#[derive(Component, Default)]
pub struct PropellerHub {
    pub current_rpm: f32,
}
