use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

// Import our modular components
use gta_game::*;
use gta_game::components::world::{MeshCache, EntityLimits};
use gta_game::systems::{SpawnValidationPlugin, DistanceCachePlugin, DistanceCacheDebugPlugin, TransformSyncPlugin};
use gta_game::setup::vehicles::{setup_simple_helicopter, setup_simple_f16};
use gta_game::setup::world::setup_dubai_noon_lighting;
use gta_game::plugins::{UnifiedWorldPlugin, RevolutionaryParallelJobPlugin};
use gta_game::systems::{PerformanceDashboardPlugin};
use gta_game::services::{initialize_simple_services, update_timing_service_system};
use gta_game::systems::{service_example_vehicle_creation, service_example_config_validation, service_example_timing_check};

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
        .init_resource::<GameConfig>()
        .init_resource::<CullingSettings>()
        .init_resource::<PerformanceStats>()
        
        .init_resource::<MeshCache>()
        .init_resource::<EntityLimits>()
        .insert_resource(ClearColor(Color::srgb(0.2, 0.8, 1.0))) // Cyan blue Dubai golden hour sky
        .insert_resource(AmbientLight {
            color: Color::srgb(1.0, 0.9, 0.7), // Warm golden ambient light
            brightness: 1800.0, // Balanced ambient for golden hour glow
            affects_lightmapped_meshes: true,
        })
        
        // Add our custom plugins
        .add_plugins(InputPlugin)         // Must be first for input processing
        .add_plugins(PlayerPlugin)
        .add_plugins(VehiclePlugin)
        .add_plugins(SpawnValidationPlugin)
        .add_plugins(DistanceCachePlugin) // Add distance caching for performance
        .add_plugins(DistanceCacheDebugPlugin)
        .add_plugins(TransformSyncPlugin) // Single source of truth for smooth transforms
        .add_plugins(VegetationLODPlugin) // Add vegetation LOD system
        //.add_plugins(BatchingPlugin) // Add optimized batching with dirty flags (disabled due to compilation issues)
        .add_plugins(PersistencePlugin)
        
        // 🚀 REVOLUTIONARY PERFORMANCE SYSTEMS - Next-generation performance
        .add_plugins(RevolutionaryParallelJobPlugin)     // 3x parallel speedup

        .add_plugins(PerformanceDashboardPlugin)         // Real-time monitoring
        
        // WORLD SYSTEM: Use unified system or old system
        // Uncomment one of these:
        // .add_plugins(WorldPlugin)              // OLD system (multiple separate streaming)
        .add_plugins(UnifiedWorldPlugin)    // NEW unified system (single coordinated streaming)
        // .add_plugins(MixedWorldPlugin)       // BOTH systems for comparison
        
        .add_plugins(UIPlugin)
        .add_plugins(WaterPlugin)
;
        
    
        
    app
        // Service initialization systems (run before everything else)
        .add_systems(Startup, initialize_simple_services)
        // Service update systems and examples
        .add_systems(Update, (
            update_timing_service_system,
            service_example_vehicle_creation,
            service_example_config_validation,
            service_example_timing_check,
        ))
        // Setup systems
        .add_systems(Startup, (
            setup_basic_world,
            setup_dubai_noon_lighting, // Add bright Dubai noon lighting
            // Aircraft with full visuals
            setup_simple_helicopter,
            setup_simple_f16,
            // Environment only
            setup_palm_trees,
            setup_npcs,
            // Starter vehicles (minimal, non-overlapping)
            setup_starter_vehicles,
            // Rest are dynamic via dynamic_content_system
        ).after(initialize_simple_services));
        

        
    app.run();
}
