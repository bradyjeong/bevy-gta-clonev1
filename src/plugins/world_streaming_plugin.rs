use bevy::prelude::*;
use crate::systems::world::{
    UnifiedWorldManager,
    unified_world_streaming_system,
    layered_generation_coordinator,
};

/// Plugin responsible for world streaming and chunk management
pub struct WorldStreamingPlugin;

impl Plugin for WorldStreamingPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<UnifiedWorldManager>()
            .add_systems(Startup, initialize_streaming_world)
            .add_systems(Update, (
                unified_world_streaming_system,
                layered_generation_coordinator,
            ).chain().in_set(crate::system_sets::GameSystemSets::WorldSetup));
    }
}

fn initialize_streaming_world(mut world_manager: ResMut<UnifiedWorldManager>) {
    world_manager.chunks.clear();
    world_manager.placement_grid.clear();
    world_manager.road_network.reset();
    #[cfg(debug_assertions)]
        println!("DEBUG: World streaming initialized!");
}
