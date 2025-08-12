use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

use crate::components::world::{MeshCache, EntityLimits};
use crate::components::{DirtyFlagsMetrics, CullingSettings, PerformanceStats};
use crate::GlobalRng;
use crate::events::*;
use crate::events::world::chunk_events::ChunkFinishedLoading;
use crate::plugins::{
    InputPlugin, PlayerPlugin, VehiclePlugin, VegetationLODPlugin, 
    PersistencePlugin, UIPlugin, WaterPlugin, UnifiedWorldPlugin, SpawnValidationPlugin
};
use crate::systems::{
    DistanceCachePlugin, DistanceCacheDebugPlugin, 
    TransformSyncPlugin, UnifiedDistanceCalculatorPlugin, UnifiedPerformancePlugin,
    parallel_physics::ParallelPhysicsConfig
};
use crate::services::GroundDetectionPlugin;
use crate::GameState;
use crate::config::GameConfig;

/// Core plugin that groups all essential game plugins and resources
/// Simplifies main.rs by organizing plugins into logical groups
pub struct GameCorePlugin;

impl Plugin for GameCorePlugin {
    fn build(&self, app: &mut App) {
        app
            // Core Bevy and Physics
            .add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "GTA Clone".into(),
                    name: Some("gta_game_main_window".into()),
                    resolution: (1280., 800.).into(),
                    position: bevy::window::WindowPosition::Centered(bevy::window::MonitorSelection::Primary),
                    resizable: true,
                    decorations: true,
                    canvas: None,
                    transparent: false,
                    focused: true,
                    visible: true,
                    mode: bevy::window::WindowMode::Windowed,
                    present_mode: bevy::window::PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            }))
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            
            // Game State and Resources
            .init_state::<GameState>()
            .init_resource::<GameConfig>()
            .init_resource::<CullingSettings>()
            .init_resource::<PerformanceStats>()
            .init_resource::<DirtyFlagsMetrics>()
            .init_resource::<MeshCache>()
            .init_resource::<EntityLimits>()
            .init_resource::<GlobalRng>()
            .init_resource::<ParallelPhysicsConfig>()
            
            // World Generation Events (Event-Driven Architecture)
            .add_event::<RequestChunkLoad>()
            .add_event::<ChunkLoaded>()
            .add_event::<ChunkFinishedLoading>()
            .add_event::<RequestChunkUnload>()
            .add_event::<ChunkUnloaded>()
            .add_event::<RequestDynamicSpawn>()
            .add_event::<DynamicContentSpawned>()
            .add_event::<RequestDynamicDespawn>()
            .add_event::<DynamicContentDespawned>()
            .add_event::<RequestSpawnValidation>()
            .add_event::<SpawnValidationResult>()
            .add_event::<RequestRoadValidation>()
            .add_event::<RoadValidationResult>()
            
            // Service Coordination Events (Replaces Direct Calls)
            .add_event::<RequestDistance>()
            .add_event::<RequestDistanceToReference>()
            .add_event::<DistanceResult>()
            .add_event::<DistanceToReferenceResult>()
            .add_event::<RequestGroundHeight>()
            .add_event::<GroundHeightResult>()
            .add_event::<RequestSpawnPositionValidation>()
            .add_event::<SpawnPositionValidationResult>()
            .insert_resource(ClearColor(Color::srgb(0.2, 0.8, 1.0)))
            .insert_resource(AmbientLight {
                color: Color::srgb(1.0, 0.9, 0.7),
                brightness: 1800.0,
                affects_lightmapped_meshes: true,
            })
            
            // Input and Player Systems
            .add_plugins((
                InputPlugin,
                PlayerPlugin,
            ))
            
            // Vehicle Systems
            .add_plugins(VehiclePlugin)
            
            // World and Environment Systems
            .add_plugins((
                VegetationLODPlugin,
                WaterPlugin,
                GroundDetectionPlugin,
                UnifiedWorldPlugin,
            ))
            
            // Distance and Performance Systems
            .add_plugins((
                SpawnValidationPlugin,
                DistanceCachePlugin,
                UnifiedDistanceCalculatorPlugin,
                DistanceCacheDebugPlugin,
                TransformSyncPlugin,
                UnifiedPerformancePlugin,
            ))
            
            // Observer Pattern Migration
            // ContentObserverPlugin already added by WorldStreamingPlugin
            
            // Persistence and UI Systems
            .add_plugins((
                PersistencePlugin,
                UIPlugin,
            ));
            
        // Debug and Instrumentation
        #[cfg(feature = "event-audit")]
        app.add_plugins(crate::debug::EventAuditPlugin);

        // Ensure window is visible and focused on startup (macOS safety)
        app.add_systems(Startup, ensure_window_visible)
            // Extra activation nudge on macOS after startup
            .add_systems(Startup, macos_activate_app)
            // Add a heartbeat to validate the event loop isn't blocked
            .add_systems(Update, heartbeat);
    }
}

fn ensure_window_visible(
    mut q: Query<&mut bevy::window::Window, With<bevy::window::PrimaryWindow>>,
) {
    if let Ok(mut w) = q.single_mut() {
        w.visible = true;
        w.focused = true;
        // Force a sane windowed mode and ensure it's on-screen
        w.mode = bevy::window::WindowMode::Windowed;
        w.present_mode = bevy::window::PresentMode::AutoVsync;
        // Nudge to a known on-screen location at a safe size
        w.resolution.set(1280.0, 800.0);
        w.position = bevy::window::WindowPosition::At(IVec2::new(100, 100));
    }
}

#[cfg(target_os = "macos")]
fn macos_activate_app(
    windows: NonSend<bevy::winit::WinitWindows>,
    q: Query<Entity, With<bevy::window::PrimaryWindow>>,
) {
    if let Ok(entity) = q.single() {
        if let Some(w) = windows.get_window(entity) {
            // Ensure the window is definitely visible and focused
            w.set_visible(true);
            w.focus_window();
            // Ask macOS to bring the app to the front just in case it launched unfocused
            let _ = w.request_user_attention(None);
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn macos_activate_app() {}

fn heartbeat(mut t: bevy::prelude::Local<f32>, time: bevy::prelude::Res<bevy::prelude::Time>) {
    *t += time.delta_secs();
    if *t >= 1.0 {
        *t = 0.0;
        bevy::prelude::info!("heartbeat: update is alive");
    }
}
