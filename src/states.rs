use bevy::prelude::*;

/// Application state machine for world generation and gameplay
/// Window loads first, then Loading state generates world, then InGame
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Loading,
    InGame,
}
