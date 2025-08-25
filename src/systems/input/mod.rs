// Legacy input modules moved to examples/legacy/
pub mod asset_based_controls;

pub use asset_based_controls::{
    LoadedVehicleControls, VehicleControlsConfig, asset_based_input_mapping_system,
    get_vehicle_control_help, load_vehicle_controls_system, process_loaded_controls_system,
};
