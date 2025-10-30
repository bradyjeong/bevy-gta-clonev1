use bevy::prelude::*;

#[derive(Event)]
pub enum SwimmingEvent {
    EnterWater {
        entity: Entity,
        depth: f32,
    },
    ExitWater {
        entity: Entity,
    },
    UpdateDepth {
        entity: Entity,
        depth: f32,
        velocity: Vec3,
    },
}
