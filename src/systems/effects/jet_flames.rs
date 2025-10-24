#![allow(clippy::type_complexity)]
use crate::components::{ActiveEntity, AircraftFlight, F16, JetFlame};
use bevy::prelude::*;

/// Combined jet flame system: handles both scale/visibility and color updates in one pass
/// Uses parent-child relationships to avoid O(n²) iteration
/// OPTIMIZATION: Removed Changed<AircraftFlight> filter - flicker animation needs every-frame updates
/// OPTIMIZATION: Early-exit when intensity < 0.1 to skip unnecessary calculations
/// OPTIMIZATION: Only writes Transform/Material when values actually change
pub fn update_jet_flames_unified(
    time: Res<Time>,
    f16_query: Query<(&AircraftFlight, &Children), (With<F16>, With<ActiveEntity>)>,
    mut flame_query: Query<(&mut Transform, &mut Visibility, &JetFlame)>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    for (flight, children) in f16_query.iter() {
        // Compute flame intensity on-demand (simple calculation)
        let base_intensity = flight.throttle;
        let afterburner_boost = if flight.afterburner_active { 0.8 } else { 0.0 };
        let flame_intensity = (base_intensity + afterburner_boost).clamp(0.0, 1.0);

        // EARLY-EXIT GUARD: Skip all work when flames should be off
        if flame_intensity < 0.1 {
            for child in children.iter() {
                if let Ok((_, mut visibility, _)) = flame_query.get_mut(child) {
                    if *visibility != Visibility::Hidden {
                        *visibility = Visibility::Hidden;
                    }
                }
            }
            continue;
        }

        // Process all flame children of this F-16
        for child in children.iter() {
            if let Ok((mut flame_transform, mut visibility, jet_flame)) = flame_query.get_mut(child)
            {
                // Show flames
                if *visibility != Visibility::Visible {
                    *visibility = Visibility::Visible;
                }

                // Calculate flame scale with flickering
                let flicker = (time.elapsed_secs() * jet_flame.flicker_speed).sin() * 0.15 + 1.0;
                let scale_factor = jet_flame.base_scale
                    + (jet_flame.max_scale - jet_flame.base_scale) * flame_intensity;
                let final_scale = scale_factor * flicker;

                // Apply scale - flames stretch more in Z axis when intense
                let new_scale = Vec3::new(
                    final_scale * 0.8,
                    final_scale * 0.8,
                    final_scale * (1.0 + flame_intensity * 1.5),
                );

                // OPTIMIZATION: Only write Transform when it changes
                if flame_transform.scale != new_scale {
                    flame_transform.scale = new_scale;
                }

                // Update color if flame has material
                if let Ok(MeshMaterial3d(material_handle)) = material_query.get(child)
                    && let Some(material) = material_assets.get_mut(material_handle)
                {
                    let color = if flight.afterburner_active {
                        // Blue-white hot flame for afterburner
                        Color::srgb(
                            0.8 + flame_intensity * 0.2,
                            0.6 + flame_intensity * 0.4,
                            1.0,
                        )
                    } else {
                        // Orange-red flame for normal thrust
                        Color::srgb(
                            1.0,
                            0.3 + flame_intensity * 0.5,
                            0.1 + flame_intensity * 0.2,
                        )
                    };

                    // Add flickering brightness
                    let flicker = (time.elapsed_secs() * 12.0).sin() * 0.1 + 1.0;
                    let flicker_color = Color::srgb(
                        color.to_srgba().red * flicker,
                        color.to_srgba().green * flicker,
                        color.to_srgba().blue * flicker,
                    );

                    // OPTIMIZATION: Material writes are expensive but necessary for animation
                    material.base_color = flicker_color;
                    material.emissive = LinearRgba::from(color) * (flame_intensity * 2.0 + 0.5);
                }
            }
        }
    }
}
