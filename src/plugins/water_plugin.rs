use crate::components::unified_water::{UnifiedWaterAsset, GlobalOcean};
use crate::systems::water::{
    load_unified_water_assets, process_loaded_unified_water_assets,
    buoyancy_system, water_drag_system,
    surface_render_system, update_water_surface_system,
    spawn_test_yacht,
};
use crate::systems::swimming::{
    swim_state_transition_system, swim_velocity_apply_system,
    swim_animation_flag_system, reset_animation_on_land_system,
    apply_prone_rotation_system,
};
use crate::game_state::GameState;

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

pub struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register RON asset loader for water regions
            .add_plugins(RonAssetPlugin::<UnifiedWaterAsset>::new(&["ron"]))
            
            // Register water assets and resources
            .init_asset::<UnifiedWaterAsset>()
            .init_resource::<GlobalOcean>()
            
            // Asset loading systems
            .add_systems(Startup, (load_unified_water_assets, spawn_test_yacht))
            .add_systems(Update, process_loaded_unified_water_assets)
            
            // Physics systems (FixedUpdate for deterministic physics)
            .add_systems(
                FixedUpdate,
                (
                    buoyancy_system, 
                    water_drag_system,
                    swim_state_transition_system,
                    swim_velocity_apply_system.run_if(in_state(GameState::Swimming)),
                ).chain(),
            )
            
            // Rendering systems (Update for smooth visuals)
            .add_systems(
                Update,
                (
                    surface_render_system, 
                    update_water_surface_system,
                    swim_animation_flag_system.run_if(in_state(GameState::Swimming)),
                    apply_prone_rotation_system, // Run always to handle return to upright
                    reset_animation_on_land_system,
                ),
            );
    }
}
