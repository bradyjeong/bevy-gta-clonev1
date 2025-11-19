use bevy::prelude::*;
use bevy_rapier3d::prelude::Collider;

use super::control_state::VehicleControlType;

#[derive(Component, Debug, Clone)]
pub struct DeckWalker {
    pub yacht: Entity,
    pub deck_anchor: Entity,
    pub last_anchor: GlobalTransform,
    pub half_extents: Vec2,
    pub foot_offset: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct LandedOnYacht {
    pub yacht: Entity,
}

#[derive(Component, Debug, Clone)]
pub struct DockedOnYacht {
    pub yacht: Entity,
    pub stored_collider: Collider,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Enterable {
    pub vehicle_type: VehicleControlType,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitPointKind {
    Deck,
    Water,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ExitPoint {
    pub kind: ExitPointKind,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct DeckWalkable;

#[derive(Component, Debug, Clone, Copy)]
pub struct Helipad;

#[derive(Component, Debug, Clone, Copy)]
pub struct DeckWalkAnchor;

#[derive(Component, Debug, Clone)]
pub struct DockingCooldown {
    pub timer: Timer,
}
