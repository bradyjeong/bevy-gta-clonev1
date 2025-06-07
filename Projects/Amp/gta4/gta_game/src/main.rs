use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

// Import our modular components
use gta_game::*;
use gta_game::systems::SkyPlugin;
use gta_game::setup::vehicles::{setup_helicopter, setup_f16};
use gta_game::plugins::{UnifiedWorldPlugin, MixedWorldPlugin};

fn main() {
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: bevy::window::PresentMode::AutoVsync, // Enable VSync for smoother framerate
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .init_state::<GameState>()
        .init_resource::<CullingSettings>()
        .init_resource::<PerformanceStats>()
        .insert_resource(ClearColor(Color::srgb(0.85, 0.9, 1.0)))
        
        // Add our custom plugins
        .add_plugins(PlayerPlugin)
        .add_plugins(VehiclePlugin)
        
        // WORLD SYSTEM: Use unified system or old system
        // Uncomment one of these:
        // .add_plugins(WorldPlugin)              // OLD system (multiple separate streaming)
        .add_plugins(UnifiedWorldPlugin)    // NEW unified system (single coordinated streaming)
        // .add_plugins(MixedWorldPlugin)       // BOTH systems for comparison
        
        .add_plugins(UIPlugin)
        .add_plugins(WaterPlugin)
        .add_plugins(SkyPlugin);
        
    // Conditionally add weather plugin
    #[cfg(feature = "weather")]
    app.add_plugins(WeatherPlugin);
        
    app
        // Setup systems
        .add_systems(Startup, (
            setup_basic_world,
            // Aircraft with full visuals
            setup_helicopter,
            setup_f16,
            // Environment only
            setup_palm_trees,
            setup_npcs,
            // Starter vehicles (minimal, non-overlapping)
            setup_starter_vehicles,
            // Rest are dynamic via dynamic_content_system
        ));
        
    // Conditionally add weather setup systems
    #[cfg(feature = "weather")]
    app.add_systems(Startup, (
        setup_weather_components,
        setup_weather_materials, 
        setup_weather_ui,
        setup_weather_environment,
    ));
    
    // Conditionally add weather update systems
    #[cfg(feature = "weather")]
    app.add_systems(Update, (
        update_weather_ui,
    ));
        
    app.run();
}
