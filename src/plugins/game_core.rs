use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy_hanabi::HanabiPlugin;
use bevy_rapier3d::prelude::*;
use std::time::Duration;

use crate::GameState;
use crate::components::world::{EntityLimits, MaterialCache, MeshCache, WorldBounds};
use crate::components::{CullingSettings, DirtyFlagsMetrics, PerformanceStats};
use crate::config::GameConfig;
use crate::plugins::{
    InputPlugin, MapPlugin, PlayerPlugin, UIPlugin, UnifiedWorldPlugin, VehiclePlugin, WaterPlugin,
};
use crate::resources::WorldRng;
use crate::services::GroundDetectionPlugin;
use crate::systems::performance::{DebugUIPlugin, PerformancePlugin, UnifiedPerformancePlugin};
use crate::systems::physics::apply_universal_physics_safeguards;
use crate::systems::player_physics_enable::enable_player_physics_next_frame;
use crate::systems::safe_active_entity::{
    active_entity_integrity_check, active_transfer_executor_system,
};
use crate::systems::world::boundaries::{aircraft_boundary_system, world_boundary_system};
use crate::systems::world::entity_limit_enforcement::enforce_entity_limits;
use crate::systems::{SpawnValidationPlugin, TransformSyncPlugin};

/// Core plugin that groups all essential game plugins and resources
/// Simplifies main.rs by organizing plugins into logical groups
pub struct GameCorePlugin;

impl Plugin for GameCorePlugin {
    fn build(&self, app: &mut App) {
        app
            // Core Bevy and Physics
            .add_plugins(
                DefaultPlugins
                    .set(WindowPlugin {
                        primary_window: Some(Window {
                            present_mode: bevy::window::PresentMode::AutoVsync,
                            ..default()
                        }),
                        ..default()
                    })
                    .set(AssetPlugin {
                        file_path: if cfg!(target_os = "macos")
                            && std::env::current_exe()
                                .map(|exe| exe.to_string_lossy().contains(".app/Contents/MacOS"))
                                .unwrap_or(false)
                        {
                            "../Resources/assets".to_string()
                        } else {
                            "assets".to_string()
                        },
                        ..default()
                    }),
            )
            // Performance optimizations: Lock physics to 60Hz fixed timestep
            .insert_resource(Time::<Fixed>::from_hz(60.0))
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .add_plugins(HanabiPlugin)
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            // Game State and Resources
            .init_state::<GameState>()
            .init_resource::<GameConfig>()
            .init_resource::<CullingSettings>()
            .init_resource::<PerformanceStats>()
            .init_resource::<DirtyFlagsMetrics>()
            .init_resource::<MeshCache>()
            .init_resource::<EntityLimits>()
            .init_resource::<WorldRng>()
            // Coordinate safety resources
            // World boundary system - initialize from config (runs before validation)
            .add_systems(
                PreStartup,
                (
                    |mut commands: Commands, config: Res<GameConfig>| {
                        let bounds = WorldBounds::from_config(&config.world);
                        commands.insert_resource(bounds);
                    },
                    |mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>| {
                        let material_cache = MaterialCache::new(&mut materials);
                        commands.insert_resource(material_cache);
                    },
                ),
            )
            // No world origin shift events needed
            .insert_resource(ClearColor(Color::srgb(0.2, 0.8, 1.0)))
            .insert_resource(AmbientLight {
                color: Color::srgb(1.0, 0.9, 0.7),
                brightness: 1800.0,
                affects_lightmapped_meshes: true,
            })
            // Input and Player Systems
            .add_plugins((InputPlugin, PlayerPlugin))
            // Vehicle Systems
            .add_plugins(VehiclePlugin)
            // World and Environment Systems
            .add_plugins((WaterPlugin, GroundDetectionPlugin, UnifiedWorldPlugin))
            // Performance and Validation Systems
            .add_plugins((
                SpawnValidationPlugin,
                TransformSyncPlugin,
                PerformancePlugin,
                UnifiedPerformancePlugin,
                DebugUIPlugin,
            ))
            // UI Systems
            .add_plugins((UIPlugin, MapPlugin))
            // Setup world root entity at startup
            // No longer need WorldRoot setup
            // Re-enable player physics before Rapier reads poses (safe vehicle exit)
            .add_systems(
                FixedUpdate,
                enable_player_physics_next_frame.before(PhysicsSet::SyncBackend),
            )
            // Movement systems run BEFORE Rapier physics step (explicit per-system ordering)
            .add_systems(
                FixedUpdate,
                (
                    crate::systems::movement::car_movement.before(PhysicsSet::SyncBackend),
                    crate::systems::movement::simple_helicopter_movement
                        .before(PhysicsSet::SyncBackend),
                    crate::systems::movement::simple_f16_movement.before(PhysicsSet::SyncBackend),
                ),
            )
            // Safeguards and boundaries run AFTER Rapier physics completes (explicit ordering + chained)
            // Throttled to 10Hz for performance
            .add_systems(
                FixedUpdate,
                (
                    apply_universal_physics_safeguards,
                    world_boundary_system,
                    aircraft_boundary_system,
                )
                    .chain()
                    .after(PhysicsSet::Writeback)
                    .run_if(on_timer(Duration::from_millis(100))),
            )
            .add_systems(
                Update,
                (
                    // Input validation catches bad positions early
                    // No position validation needed for finite world

                    // ActiveEntity safety ensures exactly one active entity
                    active_transfer_executor_system,
                    active_entity_integrity_check,
                    // Diagnostics and monitoring
                    // No floating origin diagnostics needed

                    // No sanity check system needed for finite world

                    // Entity limit enforcement with FIFO cleanup (throttled to 2Hz)
                    enforce_entity_limits.run_if(on_timer(Duration::from_millis(500))),
                )
                    .chain(),
            );

        info!("✅ Game Core Plugin loaded with physics ordering and coordinate safety");
    }
}
