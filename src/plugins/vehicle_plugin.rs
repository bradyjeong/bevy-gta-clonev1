use crate::components::vehicles::{SimpleCarSpecs, SimpleF16Specs, SimpleHelicopterSpecs};
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
    RotorWashEffect, cleanup_rotor_wash_on_helicopter_despawn, create_rotor_wash_effect,
    exhaust_effects_system, spawn_rotor_wash_particles, update_jet_flames_unified,
    update_landing_lights, update_navigation_lights, update_rotor_blur_visibility,
    update_rotor_wash_position_and_intensity,
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
            .init_asset::<SimpleCarSpecs>()
            .init_asset::<SimpleHelicopterSpecs>()
            .init_asset::<SimpleF16Specs>()
            // CRITICAL SAFEGUARDS: Run configuration validation at startup
            .add_systems(Startup, (validate_physics_config, init_rotor_wash_effect))
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
                    exhaust_effects_system,
                    // Helicopter visual enhancements
                    update_rotor_blur_visibility,
                    update_navigation_lights,
                    update_landing_lights,
                    // Rotor wash dust particles
                    spawn_rotor_wash_particles,
                    update_rotor_wash_position_and_intensity,
                ),
            )
            .add_systems(PostUpdate, cleanup_rotor_wash_on_helicopter_despawn)
            .add_systems(
                Update,
                (
                    // Visual effects - unified flame system
                    update_jet_flames_unified,
                    // PERFORMANCE MONITORING: Temporarily disabled due to static mut issues
                    // physics_performance_monitoring_system,
                    // adaptive_performance_system,
                ),
            );
    }
}

/// Initializes the rotor wash effect resource at startup.
/// Creates the effect once and caches the handle for reuse across all helicopters.
fn init_rotor_wash_effect(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let handle = create_rotor_wash_effect(&mut effects);
    commands.insert_resource(RotorWashEffect { handle });
}
