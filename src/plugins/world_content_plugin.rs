use bevy::prelude::*;
use crate::systems::world::{
    road_layer_system,
    building_layer_system,
    vehicle_layer_system,
    vegetation_layer_system,
    // dynamic_terrain_system - DISABLED: conflicts with WorldRoot coordinate shifting
};
use crate::systems::world::layered_generation::DeterministicRng;

/// Plugin responsible for generating world content (roads, buildings, vehicles, vegetation)
pub struct WorldContentPlugin;

impl Plugin for WorldContentPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<DeterministicRng>()
            .add_systems(Update, (
                road_layer_system,
                building_layer_system,
                vehicle_layer_system,
                vegetation_layer_system,
                // dynamic_terrain_system - DISABLED: terrain follows WorldRoot automatically
            ).chain());
    }
}
