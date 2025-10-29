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
    mut next_state: ResMut<NextState<AppState>>,
) {
    if specs.all_loaded(&asset_server) {
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

        #[cfg(debug_assertions)]
        {
            panic!("❌ CRITICAL: Vehicle spec assets failed to load. Check RON files!");
        }

        #[cfg(not(debug_assertions))]
        {
            error!("❌ Vehicle spec assets failed to load - gameplay will be limited");
            next_state.set(AppState::WorldGeneration);
        }
    }
}

pub fn advance_to_ingame(mut next_state: ResMut<NextState<AppState>>) {
    #[cfg(feature = "debug-ui")]
    info!("🎮 Starting gameplay");
    next_state.set(AppState::InGame);
}
