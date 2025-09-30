use crate::config::GameConfig;
use crate::resources::MaterialRegistry;
use crate::systems::world::{
    async_chunk_generation::StreamingSet, unified_world::UnifiedWorldManager,
    unified_world::unified_world_streaming_system,
};
use bevy::prelude::*;

/// Plugin responsible for world streaming and chunk management
/// Now uses async generation system for smooth 60+ FPS
pub struct WorldStreamingPlugin;

impl Plugin for WorldStreamingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (initialize_streaming_world, initialize_material_registry).chain(),
        )
        .add_systems(
            Update,
            unified_world_streaming_system.in_set(StreamingSet::Scan),
        );
    }
}

fn initialize_streaming_world(mut commands: Commands, config: Res<GameConfig>) {
    // Initialize UnifiedWorldManager with finite world configuration
    let world_manager = UnifiedWorldManager::from_config(&config);
    commands.insert_resource(world_manager);

    println!(
        "DEBUG: World streaming initialized with finite world ({}x{} chunks)!",
        config.world.total_chunks_x, config.world.total_chunks_z
    );
}

fn initialize_material_registry(mut commands: Commands) {
    // Initialize MaterialRegistry for performance optimization
    let material_registry = MaterialRegistry::new();
    commands.insert_resource(material_registry);

    println!("🏭 MATERIAL REGISTRY: Initialized for cached material reuse");
}
