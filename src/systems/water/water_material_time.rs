use crate::components::water_material::WaterMaterial;
use bevy::prelude::*;

/// Update all WaterMaterial instances with current time for wave animation
pub fn update_water_material_time_system(
    time: Res<Time>,
    mut materials: ResMut<Assets<WaterMaterial>>,
) {
    let elapsed = time.elapsed_secs();

    for (_, mat) in materials.iter_mut() {
        mat.time = elapsed;
    }
}
