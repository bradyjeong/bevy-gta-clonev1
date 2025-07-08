use bevy::prelude::*;
use bevy::text::Text;
use bevy_rapier3d::prelude::*;
use game_core::components::{ExhaustFlame, VehicleBeacon, F16};
use game_core::components::Player;
use game_core::components::WaypointText;

/// Flame effect component for jet engines
#[derive(Component)]
pub struct FlameEffect {
    pub intensity: f32,
    pub color_temperature: f32,
    pub heat_level: f32,
}

impl Default for FlameEffect {
    fn default() -> Self {
        Self {
            intensity: 1.0,
            color_temperature: 3000.0,
            heat_level: 0.0,
        }
    }
}

/// System to handle flame effects for jet engines
pub fn jet_flame_effects_system(
    mut flame_query: Query<(&mut FlameEffect, &Transform), With<F16>>,
    time: Res<Time>,
) {
    for (mut flame, transform) in flame_query.iter_mut() {
        // Calculate flame intensity based on velocity and throttle
        let velocity_magnitude = transform.translation.length();
        flame.intensity = (velocity_magnitude / 100.0).clamp(0.1, 2.0);
        
        // Animate flame temperature and heat level
        let time_factor = time.elapsed_secs() * 5.0;
        flame.color_temperature = 2500.0 + 1000.0 * (time_factor.sin() * 0.5 + 0.5);
        flame.heat_level = flame.intensity * 0.8;
        
        // Update flame visual properties based on intensity
        if flame.intensity > 1.5 {
            // High intensity afterburner effect
        } else {
            // Normal thrust flame
        }
    }
}

/// System to update flame colors based on intensity
pub fn update_flame_colors(
    flame_query: Query<(Entity, &FlameEffect), With<F16>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, flame_effect) in flame_query.iter() {
        // Color flame based on temperature and intensity
        let base_color = Color::srgb(
            1.0, 
            0.4 + flame_effect.intensity * 0.3, 
            0.1 + flame_effect.intensity * 0.2
        );
        
        // Temperature affects blue component
        let temp_factor = (flame_effect.color_temperature - 2000.0) / 2000.0;
        let [r, g, b] = base_color.to_srgba().to_u8_array_no_alpha();
        let (r, g, b) = (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
        let flame_color = Color::srgb(
            r,
            g,
            b + temp_factor * 0.3
        );
        
        // Apply color to material (implementation would depend on material setup)
    }
}

/// System to handle exhaust particle effects
pub fn exhaust_effects_system(
    mut commands: Commands,
    mut exhaust_query: Query<(Entity, &mut Transform), With<ExhaustFlame>>,
    time: Res<Time>,
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

/// System to update waypoints - shows distance to vehicles
pub fn update_waypoint_system(
    player_query: Query<&Transform, (With<Player>, Without<VehicleBeacon>)>,
    beacon_query: Query<&Transform, (With<VehicleBeacon>, Without<Player>)>,
    mut waypoint_text_query: Query<&mut Text, With<WaypointText>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
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
            text.sections[0].value = waypoint_info;
        }
    }
}

/// System to update beacon visibility with flashing effect
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
