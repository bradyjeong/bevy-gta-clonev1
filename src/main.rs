#![deny(unsafe_code)]
#![warn(clippy::expect_used)] // Warn for now, will deny after fixing remaining uses

use bevy::prelude::*;
use gta_game::plugins::{GameCorePlugin, GameSetupPlugin};

/// GTA-style open world game
/// High-level application flow:
/// 1. Initialize core systems (GameCorePlugin)
/// 2. Setup world and entities (GameSetupPlugin) 
/// 3. Run game loop
fn main() {
    App::new()
        .add_plugins(GameCorePlugin)
        .add_plugins(GameSetupPlugin)
        .run();
}
