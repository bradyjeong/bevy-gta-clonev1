use bevy::prelude::*;
use gta_game::plugins::{GameCorePlugin, GameSetupPlugin};
use gta_game::states::AppState;

/// GTA-style open world game
/// High-level application flow:
/// 1. Initialize core systems (GameCorePlugin) - Window loads
/// 2. AssetLoading state - Show splash screen and load essential assets with progress tracking
/// 3. WorldGeneration state - Generate static world (8,836 chunks)
/// 4. InGame state - Run gameplay systems
fn main() {
    App::new()
        .add_plugins(GameCorePlugin)
        .init_state::<AppState>()
        .add_plugins(GameSetupPlugin)
        .run();
}
