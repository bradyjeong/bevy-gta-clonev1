use bevy::prelude::*;
use crate::components::{F16, AircraftFlight, JetFlame, FlameEffect, ActiveEntity, ExhaustFlame, VehicleBeacon, WaypointText};
use crate::components::{Player};

pub fn update_jet_flames(
    time: Res<Time>,
    f16_query: Query<(Entity, &AircraftFlight), (With<F16>, With<ActiveEntity>)>,
    mut flame_query: Query<(&mut Transform, &mut Visibility, &FlameEffect, &JetFlame)>,
) {
    for (f16_entity, flight) in f16_query.iter() {
        for (mut flame_transform, mut visibility, flame_effect, jet_flame) in flame_query.iter_mut() {
            if flame_effect.parent_vehicle != f16_entity {
                continue;
            }

            // Calculate flame intensity based on throttle and afterburner (Oracle fix - use afterburner_active)
            let base_intensity = flight.throttle;
            let afterburner_boost = if flight.afterburner_active { 0.8 } else { 0.0 };
            let flame_intensity = (base_intensity + afterburner_boost).clamp(0.0, 1.0);

            // Hide flames when throttle is very low
            if flame_intensity < 0.1 {
                *visibility = Visibility::Hidden;
                continue;
            } else {
                *visibility = Visibility::Visible;
            }

            // Calculate flame scale with flickering
            let flicker = (time.elapsed_secs() * jet_flame.flicker_speed).sin() * 0.15 + 1.0;
            let scale_factor = jet_flame.base_scale + 
                (jet_flame.max_scale - jet_flame.base_scale) * flame_intensity;
            let final_scale = scale_factor * flicker;

            // Apply scale - flames stretch more in Z axis when intense
            flame_transform.scale = Vec3::new(
                final_scale * 0.8,
                final_scale * 0.8, 
                final_scale * (1.0 + flame_intensity * 1.5)
            );
        }
    }
}

pub fn update_flame_colors(
    time: Res<Time>,
    f16_query: Query<(Entity, &AircraftFlight), (With<F16>, With<ActiveEntity>)>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    flame_query: Query<(&MeshMaterial3d<StandardMaterial>, &FlameEffect), With<JetFlame>>,
) {
    for (f16_entity, flight) in f16_query.iter() {
        for (MeshMaterial3d(material_handle), flame_effect) in flame_query.iter() {
            if flame_effect.parent_vehicle != f16_entity {
                continue;
            }

            if let Some(material) = material_assets.get_mut(material_handle) {
                // Calculate flame color based on throttle and afterburner (Oracle fix - use afterburner_active)
                let base_intensity = flight.throttle;
                let afterburner_active = flight.afterburner_active;

                let color = if afterburner_active {
                    // Blue-white hot flame for afterburner
                    Color::srgb(
                        0.8 + base_intensity * 0.2,
                        0.6 + base_intensity * 0.4,
                        1.0
                    )
                } else {
                    // Orange-red flame for normal thrust
                    Color::srgb(
                        1.0,
                        0.3 + base_intensity * 0.5,
                        0.1 + base_intensity * 0.2
                    )
                };

                // Add flickering brightness
                let flicker = (time.elapsed_secs() * 12.0).sin() * 0.1 + 1.0;
                let flicker_color = Color::srgb(
                    color.to_srgba().red * flicker,
                    color.to_srgba().green * flicker,
                    color.to_srgba().blue * flicker,
                );
                material.base_color = flicker_color;
                material.emissive = LinearRgba::from(color) * (base_intensity * 2.0 + 0.5);
            }
        }
    }
}

// Exhaust effects system - cleans up old exhaust flames
pub fn exhaust_effects_system(
    mut commands: Commands,
    time: Res<Time>,
    mut exhaust_query: Query<(Entity, &mut Transform), With<ExhaustFlame>>,
) {
    let dt = time.delta_secs();
    
    for (entity, mut transform) in exhaust_query.iter_mut() {
        // Move exhaust particles backward and up slightly
        transform.translation += Vec3::new(0.0, 1.0, 0.0) * dt * 2.0;
        transform.scale *= 0.98; // Shrink over time
        
        // Remove exhaust flames after they've moved up or become too small
        if transform.translation.y > 3.0 || transform.scale.x < 0.1 {
            commands.entity(entity).despawn();
        }
    }
}

// Update waypoint system - shows distance to vehicles
pub fn update_waypoint_system(
    player_query: Query<&Transform, (With<Player>, Without<VehicleBeacon>)>,
    beacon_query: Query<&Transform, (With<VehicleBeacon>, Without<Player>)>,
    mut waypoint_text_query: Query<&mut Text, With<WaypointText>>,
) {
    if let Ok(player_transform) = player_query.single() {
        let player_pos = player_transform.translation;
        
        for mut text in waypoint_text_query.iter_mut() {
            let mut waypoint_info = String::new();
            
            for (i, beacon_transform) in beacon_query.iter().enumerate() {
                let distance = player_pos.distance(beacon_transform.translation);
                let direction = (beacon_transform.translation - player_pos).normalize();
                
                let vehicle_name = match i {
                    0 => "BUGATTI CHIRON",
                    1 => "HELICOPTER", 
                    2 => "F16 FIGHTER JET",
                    _ => "VEHICLE",
                };
                
                waypoint_info.push_str(&format!(
                    "{}: {:.0}m ({:.0}, {:.0})\n", 
                    vehicle_name, 
                    distance,
                    direction.x * 100.0,
                    direction.z * 100.0
                ));
            }
            
            text.0 = waypoint_info;
        }
    }
}

// Update beacon visibility system
pub fn update_beacon_visibility(
    mut beacon_query: Query<&mut Visibility, With<VehicleBeacon>>,
    time: Res<Time>,
) {
    let flash_cycle = (time.elapsed_secs() * 2.0).sin() > 0.0;
    
    for mut visibility in beacon_query.iter_mut() {
        *visibility = if flash_cycle { 
            Visibility::Visible 
        } else { 
            Visibility::Hidden 
        };
    }
}


