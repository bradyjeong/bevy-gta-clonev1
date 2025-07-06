use bevy::prelude::*;
use gameplay_sim::SimulationPlugin;
use gameplay_render::RenderPlugin;
use gameplay_ui::UiPlugin;

/// Main game plugin that orchestrates all domain plugins
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // Add domain-specific plugins in dependency order
        app.add_plugins((
            // Core simulation (no dependencies)
            SimulationPlugin,
            // Rendering (depends on simulation state)
            RenderPlugin,
            // UI (depends on both simulation and rendering state)
            UiPlugin,
        ));
    }
}
