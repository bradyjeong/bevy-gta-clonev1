use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

// Use modern modular crates
use gta_game_legacy::{GamePlugin, setup_basic_world};
use game_core::prelude::GameState;
use gameplay_sim::SimulationPlugin;
use gameplay_render::RenderPlugin;
use gameplay_ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        
        // Core game state and setup
        .init_state::<GameState>()
        .add_plugins(GamePlugin)
        
        // Modern modular plugins
        .add_plugins(SimulationPlugin)
        .add_plugins(RenderPlugin)
        .add_plugins(UiPlugin)
        
        // Basic world setup
        .add_systems(Startup, setup_basic_world)
        
        .run();
}
