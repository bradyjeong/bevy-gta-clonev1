use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use crate::systems::input::{
    InputConfig, InputManager, InputCompatLayer, VehicleControlConfig,
    process_input_system, update_input_compat_layer,
    LoadedVehicleControls, VehicleControlsConfig, load_vehicle_controls_system, process_loaded_controls_system,
    simple_input_mapping_system, input_smoothing_system, debug_control_state_system
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
        
        // Simple input systems - run in Update
        app.add_systems(Startup, load_vehicle_controls_system)
           .add_systems(Update, (
               process_loaded_controls_system,
               simple_input_mapping_system,
               input_smoothing_system.after(simple_input_mapping_system),
               debug_control_state_system,
           ));
        
        info!("Input Plugin initialized with simple control system");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::GameState;
    use bevy::input::InputPlugin as BevyInputPlugin;
    
    #[test]
    fn test_input_plugin_initialization() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            BevyInputPlugin,
        ))
        .init_state::<GameState>()
        .add_plugins(InputPlugin);
        
        // Verify resources are initialized
        assert!(app.world().get_resource::<InputConfig>().is_some());
        assert!(app.world().get_resource::<InputManager>().is_some());
        assert!(app.world().get_resource::<InputCompatLayer>().is_some());
        assert!(app.world().get_resource::<VehicleControlConfig>().is_some());
    }
}
