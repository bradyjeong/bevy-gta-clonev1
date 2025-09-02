use crate::components::water_new::{WaterRegionAsset, GlobalOcean};
use crate::systems::water::{
    load_water_regions_system, process_loaded_water_assets,
    buoyancy_system, water_drag_system,
    surface_render_system, update_water_surface_system,
    spawn_test_yacht,
};

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

pub struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register RON asset loader for water regions
            .add_plugins(RonAssetPlugin::<WaterRegionAsset>::new(&["ron"]))
            
            // Register water assets and resources
            .init_asset::<WaterRegionAsset>()
            .init_resource::<GlobalOcean>()
            
            // Asset loading systems
            .add_systems(Startup, (load_water_regions_system, spawn_test_yacht))
            .add_systems(Update, process_loaded_water_assets)
            
            // Physics systems (FixedUpdate for deterministic physics)
            .add_systems(
                FixedUpdate,
                (buoyancy_system, water_drag_system).chain(),
            )
            
            // Rendering systems (Update for smooth visuals)
            .add_systems(
                Update,
                (surface_render_system, update_water_surface_system),
            );
    }
}
