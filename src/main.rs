use bevy::prelude::*;
use gta_game::plugins::{GameCorePlugin, GameSetupPlugin};
use gta_game::states::AppState;

/// GTA-style open world game
/// High-level application flow:
/// 1. Initialize core systems (GameCorePlugin) - Window loads
/// 2. Loading state - Generate static world (8,836 chunks)
/// 3. InGame state - Run gameplay systems
fn main() {
    App::new()
        .add_plugins(GameCorePlugin) // Must come before init_state (includes DefaultPlugins)
        .init_state::<AppState>()
        .add_plugins(GameSetupPlugin)
        .run();
}
