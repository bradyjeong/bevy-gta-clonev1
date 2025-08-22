use crate::systems::world::vegetation_lod::{
    LODFrameCounter, adaptive_vegetation_lod_system, vegetation_billboard_mesh_generator,
    vegetation_billboard_system, vegetation_lod_batching_system,
    vegetation_lod_performance_monitor, vegetation_lod_system,
};
use bevy::prelude::*;
// Distance cache plugin already added in main.rs

pub struct VegetationLODPlugin;

impl Plugin for VegetationLODPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add required plugins
            // Distance cache plugin already added in main.rs
            // Add resources
            .insert_resource(LODFrameCounter::default())
            // Startup systems
            .add_systems(Startup, vegetation_billboard_mesh_generator)
            // Update systems with proper ordering
            .add_systems(
                Update,
                (
                    vegetation_lod_system,
                    vegetation_billboard_system,
                    adaptive_vegetation_lod_system,
                    vegetation_lod_performance_monitor,
                    vegetation_lod_batching_system,
                ),
            )
            // Run LOD systems less frequently to improve performance
            .add_systems(
                FixedUpdate,
                (vegetation_lod_system, vegetation_lod_performance_monitor),
            ); // 10Hz update rate
    }
}
