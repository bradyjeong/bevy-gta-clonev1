#![allow(clippy::type_complexity)]
use crate::components::{Helicopter, LandingLight, NavigationLight, NavigationLightType};
use bevy::prelude::*;

/// OPTIMIZATION: Timer needs to tick every frame, but we can early-exit if nothing needs updating
pub fn update_navigation_lights(
    time: Res<Time>,
    mut light_query: Query<(&mut NavigationLight, &mut PointLight, &GlobalTransform)>,
    player_query: Query<&GlobalTransform, With<crate::components::ActiveEntity>>,
) {
    let player_pos = player_query.iter().next().map(|t| t.translation()).unwrap_or_default();
    let has_player = player_query.iter().next().is_some();

    for (mut nav_light, mut point_light, transform) in light_query.iter_mut() {
        // Distance culling
        if has_player && transform.translation().distance(player_pos) > 500.0 {
            if point_light.intensity > 0.0 {
                point_light.intensity = 0.0;
            }
            continue;
        }

        nav_light.blink_timer.tick(time.delta());

        match nav_light.light_type {
            NavigationLightType::RedPort | NavigationLightType::GreenStarboard => {
                point_light.intensity = 50000.0;
            }
            NavigationLightType::WhiteTail => {
                if nav_light.blink_timer.just_finished() {
                    point_light.intensity = if point_light.intensity > 0.0 {
                        0.0
                    } else {
                        80000.0
                    };
                }
            }
            NavigationLightType::RedBeacon => {
                if nav_light.blink_timer.just_finished() {
                    point_light.intensity = if point_light.intensity > 0.0 {
                        0.0
                    } else {
                        100000.0
                    };
                }
            }
        }
    }
}

/// OPTIMIZATION: Or<Changed<Children>> handles initialization path for newly spawned lights
/// OPTIMIZATION: Only writes SpotLight.intensity when value differs meaningfully (>1e-3)
pub fn update_landing_lights(
    helicopter_query: Query<
        (&Transform, &Children),
        (
            With<Helicopter>,
            Or<(Changed<Transform>, Changed<Children>)>,
        ),
    >,
    children_query: Query<&Children>,
    mut landing_light_query: Query<(&LandingLight, &mut SpotLight)>,
    player_query: Query<&GlobalTransform, With<crate::components::ActiveEntity>>,
) {
    let player_pos = player_query.iter().next().map(|t| t.translation()).unwrap_or_default();
    let has_player = player_query.iter().next().is_some();

    for (helicopter_transform, helicopter_children) in helicopter_query.iter() {
        // Distance culling
        if has_player && helicopter_transform.translation.distance(player_pos) > 500.0 {
            // Set all landing lights for this heli to intensity 0
             for heli_child in helicopter_children.iter() {
                if let Ok(visual_body_children) = children_query.get(heli_child) {
                    for child in visual_body_children.iter() {
                        if let Ok((_, mut spot_light)) = landing_light_query.get_mut(child) {
                            spot_light.intensity = 0.0;
                        }
                    }
                }
            }
            continue;
        }

        let altitude = helicopter_transform.translation.y;

        // Navigate to HelicopterVisualBody children (landing lights are grandchildren now)
        for heli_child in helicopter_children.iter() {
            let Ok(visual_body_children) = children_query.get(heli_child) else {
                continue;
            };

            for child in visual_body_children.iter() {
                if let Ok((landing_light, mut spot_light)) = landing_light_query.get_mut(child) {
                    let new_intensity = if altitude < landing_light.activation_altitude {
                        let intensity_factor = 1.0 - (altitude / landing_light.activation_altitude);
                        200000.0 * intensity_factor.max(0.3)
                    } else {
                        0.0
                    };

                    if (spot_light.intensity - new_intensity).abs() > 1e-3 {
                        spot_light.intensity = new_intensity;
                    }
                }
            }
        }
    }
}
