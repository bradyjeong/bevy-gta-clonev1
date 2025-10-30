use crate::components::vehicles::{SimpleCarSpecs, SimpleF16Specs, SimpleHelicopterSpecs};
use crate::config::AssetLoadingPolicy;
use crate::resources::VehicleSpecsAssets;
use crate::states::AppState;
use bevy::prelude::*;

pub fn start_loading_vehicle_specs(mut commands: Commands, asset_server: Res<AssetServer>) {
    #[cfg(feature = "debug-ui")]
    info!("🔄 Loading vehicle specification assets...");
    let specs = VehicleSpecsAssets::load(&asset_server);
    commands.insert_resource(specs);
}

pub fn check_vehicle_specs_loaded(
    specs: Res<VehicleSpecsAssets>,
    asset_server: Res<AssetServer>,
    policy: Res<AssetLoadingPolicy>,
    mut next_state: ResMut<NextState<AppState>>,
    mut car_specs_assets: ResMut<Assets<SimpleCarSpecs>>,
    mut heli_specs_assets: ResMut<Assets<SimpleHelicopterSpecs>>,
    mut f16_specs_assets: ResMut<Assets<SimpleF16Specs>>,
) {
    if specs.all_loaded(&asset_server) {
        // Validate loaded specs
        if let Some(car_specs) = car_specs_assets.get_mut(&specs.car) {
            car_specs.validate();
        }
        if let Some(heli_specs) = heli_specs_assets.get_mut(&specs.helicopter) {
            heli_specs.validate();
        }
        if let Some(f16_specs) = f16_specs_assets.get_mut(&specs.f16) {
            f16_specs.validate();
        }

        #[cfg(feature = "debug-ui")]
        info!("✅ All vehicle specs loaded successfully");
        next_state.set(AppState::WorldGeneration);
    } else if specs.any_failed(&asset_server) {
        if matches!(
            asset_server.get_load_state(&specs.car),
            Some(bevy::asset::LoadState::Failed(_))
        ) {
            error!("❌ Failed to load: config/simple_car.ron");
        }
        if matches!(
            asset_server.get_load_state(&specs.helicopter),
            Some(bevy::asset::LoadState::Failed(_))
        ) {
            error!("❌ Failed to load: config/simple_helicopter.ron");
        }
        if matches!(
            asset_server.get_load_state(&specs.f16),
            Some(bevy::asset::LoadState::Failed(_))
        ) {
            error!("❌ Failed to load: config/simple_f16.ron");
        }
        if matches!(
            asset_server.get_load_state(&specs.yacht),
            Some(bevy::asset::LoadState::Failed(_))
        ) {
            error!("❌ Failed to load: config/simple_yacht.ron");
        }

        error!("❌ CRITICAL: Vehicle spec assets failed to load");

        if policy.fail_fast_on_missing {
            panic!("Asset loading failed in release build. Check deployment.");
        } else {
            warn!("⚠️ Using fallback defaults - this should not happen in production!");
        }

        // Insert default fallback specs instead of crashing
        if car_specs_assets.get(&specs.car).is_none() {
            car_specs_assets.insert(specs.car.id(), SimpleCarSpecs::default());
        }

        if heli_specs_assets.get(&specs.helicopter).is_none() {
            heli_specs_assets.insert(specs.helicopter.id(), SimpleHelicopterSpecs::default());
        }

        if f16_specs_assets.get(&specs.f16).is_none() {
            f16_specs_assets.insert(specs.f16.id(), SimpleF16Specs::default());
        }

        // Continue to world generation with defaults
        next_state.set(AppState::WorldGeneration);
    }
}

pub fn advance_to_ingame(mut next_state: ResMut<NextState<AppState>>) {
    #[cfg(feature = "debug-ui")]
    info!("🎮 Starting gameplay");
    next_state.set(AppState::InGame);
}
