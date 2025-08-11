use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use crate::systems::input::{
    InputConfig, InputManager, InputCompatLayer, VehicleControlConfig,
    process_input_system, update_input_compat_layer,
    LoadedVehicleControls, VehicleControlsConfig, load_vehicle_controls_system, process_loaded_controls_system,
    asset_based_input_mapping_system
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        // Register RON asset loader for vehicle controls
        app.add_plugins(RonAssetPlugin::<VehicleControlsConfig>::new(&["ron"]));
        
        // Register custom asset types
        app.init_asset::<VehicleControlsConfig>();
        
        // Initialize resources
        app.init_resource::<InputConfig>()
            .init_resource::<InputManager>()
            .init_resource::<InputCompatLayer>()
            .init_resource::<VehicleControlConfig>()
            .init_resource::<LoadedVehicleControls>();
        
        // Core input processing system - runs first in PreUpdate
        app.add_systems(PreUpdate, (
            process_input_system,
            update_input_compat_layer.after(process_input_system),
        ));
        
        // Asset-based input systems - run in Update
        app.add_systems(Startup, load_vehicle_controls_system)
           .add_systems(Update, (
               process_loaded_controls_system,
               asset_based_input_mapping_system,
           ));
        
        info!("Input Plugin initialized with asset-based control system");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::input::{input_config::InputAction, vehicle_control_config::VehicleType};
    
    #[test]
    fn test_input_plugin_components() {
        // Test that InputPlugin components can be instantiated
        let _config = InputConfig::default();
        let _manager = InputManager::default();
        let _compat = InputCompatLayer::default();
        let vehicle_config = VehicleControlConfig::default();
        
        // Verify default state is valid - test that controls are configured
        assert!(vehicle_config.get_key_for_vehicle_action(VehicleType::Walking, InputAction::Forward).is_some());
        assert!(vehicle_config.get_key_for_vehicle_action(VehicleType::Car, InputAction::Turbo).is_some());
    }
}
