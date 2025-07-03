use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

// Import our modular components
use gta_game::*;
use gta_game::components::world::{MeshCache, EntityLimits};
use gta_game::systems::{SpawnValidationPlugin, DistanceCachePlugin, DistanceCacheDebugPlugin, TransformSyncPlugin, UnifiedDistanceCalculatorPlugin};

use gta_game::setup_initial_aircraft_unified;
use gta_game::setup_initial_npcs_unified;
use gta_game::setup::world::setup_dubai_noon_lighting;
use gta_game::services::{initialize_simple_services, update_timing_service_system, GroundDetectionPlugin};
use gta_game::systems::{service_example_vehicle_creation, service_example_config_validation, service_example_timing_check, UnifiedPerformancePlugin, PerformanceIntegrationPlugin};
use gta_game::components::DirtyFlagsMetrics;

fn main() {
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
        .init_resource::<GameConfig>()
        .init_resource::<CullingSettings>()
        .init_resource::<PerformanceStats>()
        .init_resource::<DirtyFlagsMetrics>()
        
        .init_resource::<MeshCache>()
        .init_resource::<EntityLimits>()
        .insert_resource(ClearColor(Color::srgb(0.2, 0.8, 1.0)))
        .insert_resource(AmbientLight {
            color: Color::srgb(1.0, 0.9, 0.7),
            brightness: 1800.0,
            affects_lightmapped_meshes: true,
        })
        
        // Standard game plugins
        .add_plugins(InputPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(VehiclePlugin)
        .add_plugins(SpawnValidationPlugin)
        .add_plugins(DistanceCachePlugin)
        .add_plugins(UnifiedDistanceCalculatorPlugin)
        .add_plugins(DistanceCacheDebugPlugin)
        .add_plugins(TransformSyncPlugin)
        .add_plugins(VegetationLODPlugin)
        .add_plugins(PersistencePlugin)
        .add_plugins(UnifiedWorldPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(WaterPlugin)
        .add_plugins(GroundDetectionPlugin)
        
        // Unified Performance Monitoring
        .add_plugins(UnifiedPerformancePlugin)
        .add_plugins(PerformanceIntegrationPlugin);
        
    app
        // Service initialization systems
        .add_systems(Startup, initialize_simple_services)
        // Service update systems and examples
        .add_systems(Update, (
            update_timing_service_system,
            service_example_vehicle_creation,
            service_example_config_validation,
            service_example_timing_check,
        ))
        // Setup systems (split to avoid 12-system tuple limit)
        .add_systems(Startup, (
            setup_unified_entity_factory,
            setup_basic_world,
            setup_dubai_noon_lighting,
            // setup_basic_roads, // DISABLED: Conflicts with unified road system - causes darker road materials
            setup_initial_aircraft_unified,
        ).after(initialize_simple_services))
        .add_systems(Startup, (
            setup_palm_trees,
            setup_initial_npcs_unified,
            setup_initial_vehicles_unified,
        ).after(initialize_simple_services));
        
    app.run();
}
