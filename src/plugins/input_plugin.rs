use crate::systems::input::{
    LoadedVehicleControls, VehicleControlsConfig, load_vehicle_controls_system,
    process_loaded_controls_system,
};
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        // Register RON asset loader for vehicle controls
        app.add_plugins(RonAssetPlugin::<VehicleControlsConfig>::new(&["ron"]));

        // Register custom asset types
        app.init_asset::<VehicleControlsConfig>();

        // Initialize resources
        app.init_resource::<LoadedVehicleControls>();

        // Asset-based input systems only - ensure input mapping runs before consumers
        app.add_systems(Startup, load_vehicle_controls_system)
            .add_systems(Update, process_loaded_controls_system);

        info!("Input Plugin initialized with asset-based control system (legacy systems removed)");
    }
}
