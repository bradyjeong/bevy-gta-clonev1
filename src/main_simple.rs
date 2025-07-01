use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

// Import our basic components only
use gta_game::*;
use gta_game::components::world::{MeshCache, EntityLimits};
use gta_game::setup::vehicles::{setup_simple_helicopter, setup_simple_f16};
use gta_game::setup::world::setup_dubai_noon_lighting;

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
        .init_resource::<MeshCache>()
        .init_resource::<EntityLimits>()
        .insert_resource(ClearColor(Color::srgb(0.2, 0.8, 1.0)))
        .insert_resource(AmbientLight {
            color: Color::srgb(1.0, 0.9, 0.7),
            brightness: 1800.0,
            affects_lightmapped_meshes: true,
        })
        
        // STANDARD BEVY SYSTEMS ONLY - No revolutionary systems
        .add_plugins(InputPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(VehiclePlugin)
        .add_plugins(gta_game::systems::SpawnValidationPlugin)  // Need this for spawn registry
        .add_plugins(WorldPlugin)  // Use old simple world system
        .add_plugins(UIPlugin)
        .add_plugins(WaterPlugin)
        
        // Setup systems
        .add_systems(Startup, (
            setup_basic_world,
            setup_dubai_noon_lighting,
            setup_simple_helicopter,
            setup_simple_f16,
            setup_palm_trees,
            setup_npcs,
            setup_starter_vehicles,
        ));
        
    app.run();
}
