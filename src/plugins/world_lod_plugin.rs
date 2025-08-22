use bevy::prelude::*;
use crate::systems::world::{
    update_simulation_lod,
    track_simulation_lod_stats,
    SimulationLODStats,
    SimulationLODTimer,
};

/// Lightweight LOD plugin using Bevy's VisibilityRange + minimal SimulationLOD
/// Replaces heavy MasterLODCoordinator with engine-optimized rendering culling
/// and lightweight simulation throttling for 60+ FPS performance
pub struct WorldLodPlugin;

impl Plugin for WorldLodPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize simulation LOD resources
            .init_resource::<SimulationLODStats>()
            .init_resource::<SimulationLODTimer>()
            
            // Lightweight simulation LOD update (runs every 0.25s, not every frame)
            .add_systems(Update, update_simulation_lod)
            
            // Optional performance tracking (runs every 1s)
            .add_systems(Update, track_simulation_lod_stats);
        
        // Note: Rendering LOD is now handled automatically by Bevy's VisibilityRange
        // No mesh swapping systems needed - engine handles visibility culling optimally
    }
}
