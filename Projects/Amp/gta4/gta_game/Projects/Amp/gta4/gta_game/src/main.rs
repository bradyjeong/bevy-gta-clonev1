use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

// Import our modular components
use gta_game::*;
use gta_game::systems::SkyPlugin;

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
        .add_plugins(WorldPlugin)
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
            // NEW LOD VEHICLE SYSTEMS
            setup_lod_vehicles,
            setup_lod_helicopter,
            setup_lod_f16,
            // OLD SYSTEMS (commented out for comparison)
            // setup_basic_vehicles,
            // setup_helicopter,
            // setup_f16,
            setup_palm_trees,
            setup_luxury_cars,
            setup_npcs,
            // setup_buildings, // Removed - now fully dynamic
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
