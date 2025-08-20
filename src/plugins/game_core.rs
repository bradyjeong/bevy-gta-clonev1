use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

use crate::components::world::{MeshCache, EntityLimits};
use crate::components::{DirtyFlagsMetrics, CullingSettings, PerformanceStats};
use crate::plugins::{
    InputPlugin, PlayerPlugin, VehiclePlugin, VegetationLODPlugin, 
    PersistencePlugin, UIPlugin, WaterPlugin, UnifiedWorldPlugin
};
use crate::systems::{
    SpawnValidationPlugin, DistanceCachePlugin, DistanceCacheDebugPlugin, 
    TransformSyncPlugin, UnifiedDistanceCalculatorPlugin, UnifiedPerformancePlugin,
    
    // Coordinate safety systems
    WorldOffset, FloatingOriginConfig, WorldOriginShifted, setup_world_root,
    seamless_world_rebase_system, update_physics_after_origin_shift, floating_origin_diagnostics, world_sanity_check_system, 
    world_shift_special_cases_system, ActiveEntityTransferred, active_transfer_executor_system, 
    active_entity_integrity_check, validate_streaming_position
};
use crate::systems::physics::apply_universal_physics_safeguards;
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
            .add_plugins(DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: bevy::window::PresentMode::Fifo,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: if cfg!(target_os = "macos") && std::env::current_exe()
                        .map(|exe| exe.to_string_lossy().contains(".app/Contents/MacOS"))
                        .unwrap_or(false) {
                        "../Resources/assets".to_string()
                    } else {
                        "assets".to_string()
                    },
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
            
            // Coordinate safety resources 
            .init_resource::<WorldOffset>()
            .init_resource::<FloatingOriginConfig>()
            .add_event::<ActiveEntityTransferred>()
            .add_event::<WorldOriginShifted>()
            
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
            
            // Persistence and UI Systems
            .add_plugins((
                PersistencePlugin,
                UIPlugin,
            ))
            
            // Setup world root entity at startup
            .add_systems(Startup, setup_world_root)
            
            // Coordinate safety systems with seamless world shifting
            // Seamless world rebase runs BEFORE physics simulation
            .add_systems(PreUpdate, (
                seamless_world_rebase_system,
                update_physics_after_origin_shift,
            ).chain())
            
            .add_systems(FixedUpdate, (
                // Universal physics safeguards run AFTER Rapier physics step
                apply_universal_physics_safeguards,
                
                // Special cases system handles entities with separate position data
                world_shift_special_cases_system,
            ).chain())
            
            .add_systems(Update, (
                // Input validation catches bad positions early
                validate_streaming_position,
                
                // ActiveEntity safety ensures exactly one active entity  
                active_transfer_executor_system,
                active_entity_integrity_check,
                
                // Diagnostics and monitoring
                floating_origin_diagnostics,
                
                // Safety system to catch any remaining orphaned entities
                world_sanity_check_system,
            ).chain().in_set(crate::system_sets::GameSystemSets::ServiceUpdates));
        
        info!("✅ Game Core Plugin loaded with complete coordinate safety and infinite world support");
    }
}
