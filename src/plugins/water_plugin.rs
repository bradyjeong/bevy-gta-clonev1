use crate::components::SwimmingEvent;
use crate::components::unified_water::UnifiedWaterAsset;
use crate::components::water::YachtSpecs;
use crate::components::water_material::WaterMaterial;
use crate::game_state::GameState;
use crate::states::AppState;
use crate::systems::debug_docked_heli::audit_docked_helicopter_movement;
use crate::systems::effects::boat_wake::{
    BoatWakeEffect, cleanup_boat_wake_on_despawn, create_boat_wake_effect,
    spawn_boat_wake_particles, update_boat_wake_intensity,
};
use crate::systems::movement::{
    boat_animation_system, simple_yacht_movement, spool_docked_helicopter_rpm,
};
use crate::systems::swimming::{
    apply_prone_rotation_system, apply_swimming_state, detect_swimming_conditions,
    emergency_swim_exit_system, reset_animation_on_land_system, swim_animation_flag_system,
    swim_velocity_apply_system,
};
use crate::systems::water::{
    load_unified_water_assets, process_loaded_unified_water_assets, simple_yacht_buoyancy,
    spawn_test_yacht, surface_render_system, update_water_material_time_system,
    update_water_region_cache, update_water_surface_system, water_physics_system,
};
use crate::systems::yacht_exit::{
    deck_walk_movement_system, heli_landing_detection_system, helicopter_undock_trigger_system,
    tick_docking_cooldown_system, yacht_board_from_deck_system, yacht_exit_system,
};
use bevy_hanabi::prelude::*;

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
            .add_systems(Startup, (load_unified_water_assets, spawn_test_yacht))
            .add_systems(OnEnter(AppState::InGame), init_boat_wake_effect)
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
            .add_systems(Update, boat_animation_system)
            .add_systems(
                Update,
                (
                    spawn_boat_wake_particles.run_if(resource_exists::<BoatWakeEffect>),
                    update_boat_wake_intensity.run_if(resource_exists::<BoatWakeEffect>),
                ),
            )
            .add_systems(
                FixedUpdate,
                yacht_board_from_deck_system.before(PhysicsSet::SyncBackend),
            )
            .add_systems(
                Update,
                (
                    yacht_exit_system.after(crate::plugins::input_plugin::InputProcessingSet),
                    deck_walk_movement_system,
                    heli_landing_detection_system,
                    helicopter_undock_trigger_system,
                    tick_docking_cooldown_system,
                    spool_docked_helicopter_rpm,
                ),
            )
            .add_systems(
                PostUpdate,
                (
                    audit_docked_helicopter_movement,
                    cleanup_boat_wake_on_despawn,
                ),
            )
            .add_systems(OnExit(AppState::InGame), cleanup_boat_wake_resource);
    }
}

fn init_boat_wake_effect(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let handle = create_boat_wake_effect(&mut effects);
    commands.insert_resource(BoatWakeEffect { handle });
}

fn cleanup_boat_wake_resource(
    mut commands: Commands,
    wake: Option<Res<BoatWakeEffect>>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    if let Some(wake) = wake {
        effects.remove(wake.handle.id());
    }
    commands.remove_resource::<BoatWakeEffect>();
}
