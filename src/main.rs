use bevy::prelude::*;
use gta_game::plugins::{GameCorePlugin, GameSetupPlugin};
#[cfg(feature = "simple_render_culler")]
use gta_game::systems::rendering::SimpleRenderCullerPlugin;

/// GTA-style open world game
/// High-level application flow:
/// 1. Initialize core systems (GameCorePlugin)
/// 2. Setup world and entities (GameSetupPlugin) 
/// 3. Run game loop
fn main() {
    let mut app = App::new();
    app.add_plugins(GameCorePlugin)
        .add_plugins(GameSetupPlugin);
    
    #[cfg(feature = "simple_render_culler")]
    app.add_plugins(SimpleRenderCullerPlugin);
    
    app.run();
}
