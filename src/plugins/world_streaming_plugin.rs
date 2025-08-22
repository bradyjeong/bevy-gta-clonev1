use bevy::prelude::*;
use crate::systems::world::{
    UnifiedWorldManager,
    unified_world_streaming_system,
    layered_generation_coordinator,
};
use crate::config::GameConfig;

/// Plugin responsible for world streaming and chunk management
pub struct WorldStreamingPlugin;

impl Plugin for WorldStreamingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, initialize_streaming_world)
            .add_systems(Update, (
                unified_world_streaming_system,
                layered_generation_coordinator,
            ).chain());
    }
}

fn initialize_streaming_world(
    mut commands: Commands,
    config: Res<GameConfig>,
) {
    // Initialize UnifiedWorldManager with finite world configuration
    let world_manager = UnifiedWorldManager::from_config(&config);
    commands.insert_resource(world_manager);
    
    println!("DEBUG: World streaming initialized with finite world ({}x{} chunks)!", 
             config.world.total_chunks_x, config.world.total_chunks_z);
}
