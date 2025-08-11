use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

use crate::components::world::{MeshCache, EntityLimits};
use crate::components::{DirtyFlagsMetrics, CullingSettings, PerformanceStats};
use crate::GlobalRng;
use crate::events::*;
use crate::plugins::{
    InputPlugin, PlayerPlugin, VehiclePlugin, VegetationLODPlugin, 
    PersistencePlugin, UIPlugin, WaterPlugin, UnifiedWorldPlugin
};
use crate::systems::{
    SpawnValidationPlugin, DistanceCachePlugin, DistanceCacheDebugPlugin, 
    TransformSyncPlugin, UnifiedDistanceCalculatorPlugin, UnifiedPerformancePlugin
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
                    present_mode: bevy::window::PresentMode::Fifo,
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
            
            // World Generation Events (Event-Driven Architecture)
            .add_event::<RequestChunkLoad>()
            .add_event::<ChunkLoaded>()
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
    }
}
