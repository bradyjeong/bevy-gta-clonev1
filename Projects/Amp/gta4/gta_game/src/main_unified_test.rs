use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

// Test the unified world system
use gta_game::*;
use gta_game::systems::SkyPlugin;
use gta_game::setup::vehicles::{setup_helicopter, setup_f16};
use gta_game::plugins::{UnifiedWorldPlugin, PlayerPlugin, VehiclePlugin, UIPlugin, WaterPlugin};

fn main() {
    println!("TESTING UNIFIED WORLD SYSTEM");
    
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: bevy::window::PresentMode::AutoVsync,
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
        
        // Use the NEW unified world plugin instead of the old WorldPlugin
        .add_plugins(PlayerPlugin)
        .add_plugins(VehiclePlugin)
        .add_plugins(UnifiedWorldPlugin) // NEW UNIFIED SYSTEM
        .add_plugins(UIPlugin)
        .add_plugins(WaterPlugin)
        .add_plugins(SkyPlugin);
        
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
            // Test the unified system
            log_unified_system_status,
        ));
        
    app.run();
}

fn log_unified_system_status(
    world_manager: Res<UnifiedWorldManager>,
) {
    println!("=== UNIFIED WORLD SYSTEM STATUS ===");
    println!("Chunks loaded: {}", world_manager.chunks.len());
    println!("Road network roads: {}", world_manager.road_network.roads.len());
    println!("Streaming radius: {}m", UNIFIED_STREAMING_RADIUS);
    println!("Chunk size: {}m", UNIFIED_CHUNK_SIZE);
    println!("Max chunks per frame: {}", world_manager.max_chunks_per_frame);
    println!("===================================");
}
