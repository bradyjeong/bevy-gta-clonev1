use crate::components::SwimmingEvent;
use crate::components::unified_water::UnifiedWaterAsset;
use crate::components::water::YachtSpecs;
use crate::components::water_material::WaterMaterial;
use crate::game_state::GameState;
use crate::states::AppState;
use crate::systems::movement::{propeller_spin_system, simple_yacht_movement};
use crate::systems::swimming::{
    apply_prone_rotation_system, apply_swimming_state, detect_swimming_conditions,
    emergency_swim_exit_system, reset_animation_on_land_system, swim_animation_flag_system,
    swim_velocity_apply_system,
};
use crate::systems::water::{
    cleanup_yacht_effects, cleanup_yacht_particle_entities,
    cleanup_yacht_particles_on_despawn, load_unified_water_assets,
    process_loaded_unified_water_assets, setup_yacht_effects, simple_yacht_buoyancy,
    spawn_bow_splash, spawn_or_update_wake_foam, spawn_prop_wash, spawn_test_yacht,
    surface_render_system, update_water_material_time_system, update_water_region_cache,
    update_water_surface_system, water_physics_system,
};
use crate::systems::yacht_exit::{
    deck_walk_movement_system, heli_landing_detection_system, sync_landed_helicopter_with_yacht,
    yacht_board_from_deck_system, yacht_exit_system,
};

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_rapier3d::prelude::*;

#[derive(SystemSet, Hash, Eq, PartialEq, Clone, Debug)]
pub enum WaterSystemSet {
    UpdateCache,
    Physics,
    Effects,
}

pub struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<UnifiedWaterAsset>::new(&["ron"]))
            .add_plugins(RonAssetPlugin::<YachtSpecs>::new(&["ron"]))
            .add_plugins(MaterialPlugin::<WaterMaterial>::default())
            .init_asset::<UnifiedWaterAsset>()
            .init_asset::<YachtSpecs>()
            .add_event::<SwimmingEvent>()
            .configure_sets(
                FixedUpdate,
                (
                    WaterSystemSet::UpdateCache,
                    WaterSystemSet::Physics,
                    WaterSystemSet::Effects,
                )
                    .chain()
                    .before(PhysicsSet::StepSimulation),
            )
            .add_systems(
                Startup,
                (
                    load_unified_water_assets,
                    spawn_test_yacht,
                ),
            )
            .add_systems(OnEnter(AppState::InGame), setup_yacht_effects)
            .add_systems(Update, process_loaded_unified_water_assets)
            .add_systems(
                FixedUpdate,
                update_water_region_cache.in_set(WaterSystemSet::UpdateCache),
            )
            .add_systems(
                FixedUpdate,
                (water_physics_system, simple_yacht_buoyancy)
                    .chain()
                    .in_set(WaterSystemSet::Physics),
            )
            .add_systems(
                FixedUpdate,
                (
                    simple_yacht_movement,
                    detect_swimming_conditions,
                    apply_swimming_state,
                    swim_velocity_apply_system.run_if(in_state(GameState::Swimming)),
                    sync_landed_helicopter_with_yacht,
                )
                    .chain()
                    .in_set(WaterSystemSet::Effects),
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
                    emergency_swim_exit_system,
                ),
            )
            .add_systems(
                Update,
                (
                    spawn_or_update_wake_foam,
                    spawn_bow_splash,
                    spawn_prop_wash,
                    propeller_spin_system,
                ),
            )
            .add_systems(PostUpdate, cleanup_yacht_particles_on_despawn)
            .add_systems(
                Update,
                (
                    yacht_exit_system.after(crate::plugins::input_plugin::InputProcessingSet),
                    yacht_board_from_deck_system,
                    deck_walk_movement_system,
                    heli_landing_detection_system,
                ),
            )
            .add_systems(
                OnExit(AppState::InGame),
                (cleanup_yacht_particle_entities, cleanup_yacht_effects),
            );
    }
}
