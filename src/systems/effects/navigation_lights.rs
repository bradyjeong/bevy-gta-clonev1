use crate::components::{Helicopter, LandingLight, NavigationLight, NavigationLightType};
use bevy::prelude::*;

pub fn update_navigation_lights(
    time: Res<Time>,
    mut light_query: Query<(&mut NavigationLight, &mut PointLight)>,
) {
    for (mut nav_light, mut point_light) in light_query.iter_mut() {
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

pub fn update_landing_lights(
    helicopter_query: Query<(&Transform, &Children), With<Helicopter>>,
    mut landing_light_query: Query<(&LandingLight, &mut SpotLight)>,
) {
    for (helicopter_transform, children) in helicopter_query.iter() {
        let altitude = helicopter_transform.translation.y;

        for child in children.iter() {
            if let Ok((landing_light, mut spot_light)) = landing_light_query.get_mut(child) {
                if altitude < landing_light.activation_altitude {
                    let intensity_factor = 1.0 - (altitude / landing_light.activation_altitude);
                    spot_light.intensity = 200000.0 * intensity_factor.max(0.3);
                } else {
                    spot_light.intensity = 0.0;
                }
            }
        }
    }
}
