use bevy::prelude::*;

/// Represents the current state of the game's player character.
///
/// This enum defines the different movement and interaction modes available to the player.
/// Each state affects how the player's input is processed and which systems are active.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize)]
pub enum GameState {
    /// Player is on foot and can walk around the world.
    #[default]
    Walking,
    /// Player is operating a ground vehicle (car, motorcycle, etc.).
    Driving,
    /// Player is operating a civilian aircraft (helicopter, small plane, etc.).
    Flying,
    /// Player is operating a military jet aircraft (F16, etc.).
    Jetting, // New state for F16 flying
}
