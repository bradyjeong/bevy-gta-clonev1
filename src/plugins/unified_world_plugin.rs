use crate::config::GameConfig;
use crate::factories::material_factory::initialize_material_factory;
use crate::plugins::{
    PhysicsActivationPlugin, StaticWorldGenerationPlugin, WorldDebugPlugin, WorldNpcPlugin,
};
use crate::resources::MaterialRegistry;
use crate::states::AppState;
use crate::systems::world::unified_world::UnifiedWorldManager;
use bevy::prelude::*;

/// Simplified unified world plugin - now uses static generation at startup
/// Generates entire 12km x 12km world during Loading state
pub struct UnifiedWorldPlugin;

impl Plugin for UnifiedWorldPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize world manager and resources EARLY (PreStartup, before OnEnter(Loading))
            .add_systems(
                PreStartup,
                (initialize_world_manager, initialize_material_registry).chain(),
            )
            // Add world generation and gameplay plugins
            .add_plugins(StaticWorldGenerationPlugin) // Static generation in Loading state
            .add_plugins(PhysicsActivationPlugin) // GTA-style dynamic physics activation
            .add_plugins(WorldNpcPlugin)
            .add_plugins(WorldDebugPlugin)
            // Initialize material factory
            .add_systems(Startup, initialize_material_factory)
            // Cleanup resources on game exit
            .add_systems(OnExit(AppState::InGame), cleanup_world_resources);
    }
}

fn initialize_world_manager(mut commands: Commands, config: Res<GameConfig>) {
    let world_manager = UnifiedWorldManager::from_config(&config);
    commands.insert_resource(world_manager);

    #[cfg(feature = "debug-ui")]
    info!(
        "World manager initialized: {}x{} chunks ({}km x {}km)",
        config.world.total_chunks_x,
        config.world.total_chunks_z,
        config.world.map_size / 1000.0,
        config.world.map_size / 1000.0
    );
}

fn initialize_material_registry(mut commands: Commands) {
    let material_registry = MaterialRegistry::new();
    commands.insert_resource(material_registry);
    #[cfg(feature = "debug-ui")]
    info!("Material registry initialized for cached material reuse");
}

fn cleanup_world_resources(mut commands: Commands) {
    commands.remove_resource::<UnifiedWorldManager>();
    commands.remove_resource::<MaterialRegistry>();
    #[cfg(feature = "debug-ui")]
    info!("World resources cleaned up");
}
