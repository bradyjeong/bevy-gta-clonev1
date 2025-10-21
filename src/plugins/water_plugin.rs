use crate::components::unified_water::UnifiedWaterAsset;
use crate::components::water::{WaterSurface, YachtSpecs};
use crate::components::water_material::WaterMaterial;
use crate::game_state::GameState;
use crate::systems::movement::{propeller_spin_system, simple_yacht_movement};
use crate::systems::swimming::{
    apply_prone_rotation_system, reset_animation_on_land_system, swim_animation_flag_system,
    swim_state_transition_system, swim_velocity_apply_system,
};
use crate::systems::water::{
    buoyancy_system, initialize_yacht_wake_trail, load_unified_water_assets,
    process_loaded_unified_water_assets, setup_yacht_effects, simple_yacht_buoyancy,
    spawn_bow_splash, spawn_prop_wash, spawn_test_yacht, spawn_yacht_wake_trail,
    surface_render_system, update_wake_trail_points, update_water_material_time_system,
    update_water_surface_system, water_drag_system,
};
use crate::systems::yacht_exit::{
    deck_walk_movement_system, heli_landing_detection_system, yacht_board_from_deck_system,
    yacht_exit_system,
};

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

pub struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<UnifiedWaterAsset>::new(&["ron"]))
            .add_plugins(RonAssetPlugin::<YachtSpecs>::new(&["ron"]))
            .add_plugins(MaterialPlugin::<WaterMaterial>::default())
            .init_asset::<UnifiedWaterAsset>()
            .init_asset::<YachtSpecs>()
            .init_resource::<WaterSurface>()
            .add_systems(
                Startup,
                (
                    load_unified_water_assets,
                    spawn_test_yacht,
                    setup_yacht_effects,
                ),
            )
            .add_systems(
                Update,
                (
                    process_loaded_unified_water_assets,
                    initialize_yacht_wake_trail,
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    buoyancy_system,
                    water_drag_system,
                    simple_yacht_buoyancy,
                    simple_yacht_movement,
                    swim_state_transition_system,
                    swim_velocity_apply_system.run_if(in_state(GameState::Swimming)),
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    surface_render_system,
                    update_water_surface_system,
                    update_water_material_time_system,
                    swim_animation_flag_system.run_if(in_state(GameState::Swimming)),
                    apply_prone_rotation_system,
                    reset_animation_on_land_system,
                ),
            )
            .add_systems(
                Update,
                (
                    spawn_yacht_wake_trail,
                    update_wake_trail_points,
                    spawn_bow_splash,
                    spawn_prop_wash,
                    propeller_spin_system,
                ),
            )
            .add_systems(
                Update,
                (
                    // CRITICAL: Run yacht_exit_system AFTER input processing
                    yacht_exit_system.after(crate::plugins::input_plugin::InputProcessingSet),
                    yacht_board_from_deck_system,
                    deck_walk_movement_system,
                    heli_landing_detection_system,
                ),
            );
    }
}
