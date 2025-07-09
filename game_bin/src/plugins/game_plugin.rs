use bevy::prelude::*;
use gameplay_sim::SimulationPlugin;
use gameplay_render::RenderPlugin;
use game_core::config::GameConfig;

/// Main game plugin that orchestrates all domain plugins
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // Initialize game configuration
        app.insert_resource(GameConfig::default());
        
        // Add domain-specific plugins in dependency order
        app.add_plugins((
            // Core simulation (no dependencies)
            SimulationPlugin,
            // Rendering (depends on simulation state)
            RenderPlugin,
            // UI plugin not available in game_bin dependencies
        ));
    }
}
