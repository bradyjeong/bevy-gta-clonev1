use crate::components::vehicles::{
    SimpleCarSpecs, SimpleF16Specs, SimpleHelicopterSpecs, VehiclePhysicsConfig,
};
use crate::states::AppState;
use crate::systems::camera_car::car_camera_system;
use crate::systems::camera_f16::f16_camera_system;
use crate::systems::camera_helicopter::helicopter_camera_system;
use crate::systems::camera_yacht::yacht_camera_system;
use crate::systems::movement::rotate_helicopter_rotors;
use crate::systems::setup::on_f16_spawned;
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
// Complex aircraft systems moved to examples/complex_aircraft_physics.rs
use crate::systems::effects::{
    AfterburnerFlameEffect, RotorWashEffect, cleanup_afterburner_on_f16_despawn,
    cleanup_afterburner_particle_entities, cleanup_rotor_wash_on_helicopter_despawn,
    cleanup_rotor_wash_particle_entities, create_afterburner_flame_effect,
    create_rotor_wash_effect, ensure_afterburner_for_existing_f16s,
    ensure_rotor_wash_for_existing_helicopters, spawn_afterburner_particles,
    spawn_rotor_wash_particles, update_afterburner_position_and_intensity,
    update_jet_flames_unified, update_landing_lights, update_navigation_lights,
    update_rotor_blur_visibility, update_rotor_wash_position_and_intensity,
};
use crate::systems::safety::validate_physics_config;
use bevy_hanabi::prelude::*;
// LOD system replaced with Bevy's VisibilityRange + simulation_lod
// use crate::systems::configuration_validation_system; // DISABLED - conflicts with Rapier

pub struct VehiclePlugin;

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app
            // Asset-driven vehicle specs (following YachtSpecs pattern)
            .add_plugins(RonAssetPlugin::<SimpleCarSpecs>::new(&["ron"]))
            .add_plugins(RonAssetPlugin::<SimpleHelicopterSpecs>::new(&["ron"]))
            .add_plugins(RonAssetPlugin::<SimpleF16Specs>::new(&["ron"]))
            .add_plugins(RonAssetPlugin::<VehiclePhysicsConfig>::new(&["ron"]))
            .init_asset::<SimpleCarSpecs>()
            .init_asset::<SimpleHelicopterSpecs>()
            .init_asset::<SimpleF16Specs>()
            .init_asset::<VehiclePhysicsConfig>()
            // CRITICAL SAFEGUARDS: Run configuration validation at startup
            .add_systems(Startup, validate_physics_config)
            .add_systems(
                OnEnter(AppState::InGame),
                (
                    init_rotor_wash_effect,
                    ensure_rotor_wash_for_existing_helicopters,
                    init_afterburner_effect,
                    ensure_afterburner_for_existing_f16s,
                )
                    .chain(),
            )
            // Observer for F16 setup when specs are added
            .add_observer(on_f16_spawned)
            .add_systems(
                Update,
                (
                    // REMOVED: bounds_safety_system and diagnostics - finite world eliminates need

                    // LOD now handled by Bevy's VisibilityRange automatically

                    // Movement systems moved to FixedUpdate in game_core.rs
                    // car_movement.run_if(in_state(GameState::Driving)),
                    // simple_helicopter_movement.run_if(in_state(GameState::Flying)),
                    // simple_f16_movement.run_if(in_state(GameState::Jetting)),
                    // Camera systems for smooth vehicle following
                    car_camera_system,
                    helicopter_camera_system,
                    f16_camera_system,
                    yacht_camera_system,
                    // Visual rotor animation for helicopters
                    rotate_helicopter_rotors,
                    // Helicopter visual enhancements
                    update_rotor_blur_visibility,
                    update_navigation_lights,
                    update_landing_lights,
                    // Rotor wash dust particles (only when resource exists)
                    spawn_rotor_wash_particles.run_if(resource_exists::<RotorWashEffect>),
                    update_rotor_wash_position_and_intensity
                        .run_if(resource_exists::<RotorWashEffect>),
                    // Afterburner flame particles (only when resource exists)
                    spawn_afterburner_particles.run_if(resource_exists::<AfterburnerFlameEffect>),
                    update_afterburner_position_and_intensity
                        .run_if(resource_exists::<AfterburnerFlameEffect>),
                ),
            )
            .add_systems(
                PostUpdate,
                (
                    cleanup_rotor_wash_on_helicopter_despawn,
                    cleanup_afterburner_on_f16_despawn,
                ),
            )
            .add_systems(
                Update,
                (
                    // Visual effects - unified flame system
                    update_jet_flames_unified,
                    // PERFORMANCE MONITORING: Temporarily disabled due to static mut issues
                    // physics_performance_monitoring_system,
                    // adaptive_performance_system,
                ),
            )
            .add_systems(
                OnExit(AppState::InGame),
                (
                    cleanup_rotor_wash_particle_entities,
                    cleanup_rotor_wash_effect,
                    cleanup_afterburner_particle_entities,
                    cleanup_afterburner_effect,
                )
                    .chain(),
            );
    }
}

/// Initializes the rotor wash effect resource at startup.
/// Creates the effect once and caches the handle for reuse across all helicopters.
fn init_rotor_wash_effect(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let handle = create_rotor_wash_effect(&mut effects);
    commands.insert_resource(RotorWashEffect { handle });
}

fn cleanup_rotor_wash_effect(
    mut commands: Commands,
    rotor: Option<Res<RotorWashEffect>>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    if let Some(rotor) = rotor {
        // Remove asset to prevent CPU-side asset accumulation across re-entries
        effects.remove(rotor.handle.id());
    }
    commands.remove_resource::<RotorWashEffect>();
    #[cfg(feature = "debug-ui")]
    info!("Rotor wash effect cleaned up");
}

/// Initializes the afterburner flame effect resource at startup.
/// Creates the effect once and caches the handle for reuse across all F16s.
fn init_afterburner_effect(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let handle = create_afterburner_flame_effect(&mut effects);
    commands.insert_resource(AfterburnerFlameEffect { handle });
}

fn cleanup_afterburner_effect(
    mut commands: Commands,
    afterburner: Option<Res<AfterburnerFlameEffect>>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    if let Some(afterburner) = afterburner {
        effects.remove(afterburner.handle.id());
    }
    commands.remove_resource::<AfterburnerFlameEffect>();
    #[cfg(feature = "debug-ui")]
    info!("Afterburner effect cleaned up");
}
