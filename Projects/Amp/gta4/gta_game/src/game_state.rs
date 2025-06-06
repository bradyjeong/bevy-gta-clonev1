use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Walking,
    Driving,
    Flying,
    Jetting, // New state for F16 flying
}
