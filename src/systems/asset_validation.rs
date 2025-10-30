use bevy::prelude::*;

const REQUIRED_ASSET_PATHS: &[&str] = &[
    "config/simple_car.ron",
    "config/simple_helicopter.ron",
    "config/simple_f16.ron",
    "config/simple_yacht.ron",
    "config/vehicle_controls.ron",
    "config/vehicle_physics.ron",
];

pub fn validate_asset_files_exist() {
    for path in REQUIRED_ASSET_PATHS {
        let full_path = format!("assets/{path}");
        if !std::path::Path::new(&full_path).exists() {
            error!("Missing required asset file: {}", full_path);
        } else {
            info!("Found asset: {}", path);
        }
    }
}
