use bevy::prelude::*;

use super::control_state::VehicleControlType;

#[derive(Component, Debug, Clone, Copy)]
pub struct DeckWalker {
    pub yacht: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct LandedOnYacht {
    pub yacht: Entity,
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
