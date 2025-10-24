use crate::components::vehicles::{SimpleCarSpecs, SimpleF16Specs, SimpleHelicopterSpecs};
use crate::components::water::YachtSpecs;
use bevy::prelude::*;

#[derive(Resource)]
pub struct VehicleSpecsAssets {
    pub car: Handle<SimpleCarSpecs>,
    pub helicopter: Handle<SimpleHelicopterSpecs>,
    pub f16: Handle<SimpleF16Specs>,
    pub yacht: Handle<YachtSpecs>,
}

impl VehicleSpecsAssets {
    pub fn load(asset_server: &AssetServer) -> Self {
        Self {
            car: asset_server.load("config/simple_car.ron"),
            helicopter: asset_server.load("config/simple_helicopter.ron"),
            f16: asset_server.load("config/simple_f16.ron"),
            yacht: asset_server.load("config/simple_yacht.ron"),
        }
    }

    pub fn all_loaded(&self, asset_server: &AssetServer) -> bool {
        use bevy::asset::LoadState;

        let states = [
            asset_server.get_load_state(&self.car),
            asset_server.get_load_state(&self.helicopter),
            asset_server.get_load_state(&self.f16),
            asset_server.get_load_state(&self.yacht),
        ];

        states.iter().all(|s| matches!(s, Some(LoadState::Loaded)))
    }

    pub fn any_failed(&self, asset_server: &AssetServer) -> bool {
        use bevy::asset::LoadState;

        let states = [
            asset_server.get_load_state(&self.car),
            asset_server.get_load_state(&self.helicopter),
            asset_server.get_load_state(&self.f16),
            asset_server.get_load_state(&self.yacht),
        ];

        states
            .iter()
            .any(|s| matches!(s, Some(LoadState::Failed(_))))
    }
}
