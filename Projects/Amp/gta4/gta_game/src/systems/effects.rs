use bevy::prelude::*;
use crate::components::{ExhaustFlame, VehicleBeacon, ControlsText, ControlsDisplay, WaypointText, Player};
use crate::game_state::GameState;

pub fn exhaust_effects_system(
    mut commands: Commands,
    time: Res<Time>,
    mut exhaust_query: Query<(Entity, &mut Transform), With<ExhaustFlame>>,
) {
    let dt = time.delta_secs();
    
    for (entity, mut transform) in exhaust_query.iter_mut() {
        // Move exhaust particles upward and backward
        transform.translation += Vec3::new(0.0, 2.0, -1.0) * dt;
        
        // Fade and shrink over time
        transform.scale *= 0.95;
        
        // Remove when too small or moved too far
        if transform.scale.x < 0.1 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn update_beacon_visibility(
    mut beacon_query: Query<&mut Visibility, With<VehicleBeacon>>,
    time: Res<Time>,
) {
    let flash_rate = 2.0; // Flashes per second
    let visible = (time.elapsed_secs() * flash_rate * 2.0 * std::f32::consts::PI).sin() > 0.0;
    
    for mut visibility in beacon_query.iter_mut() {
        *visibility = if visible { Visibility::Visible } else { Visibility::Hidden };
    }
}

pub fn controls_ui_system(
    state: Res<State<GameState>>,
    mut controls_query: Query<&mut Text, (With<ControlsText>, With<ControlsDisplay>)>,
) {
    let Ok(mut text) = controls_query.single_mut() else { return; };
    
    match **state {
        GameState::Walking => {
            **text = "CONTROLS - Walking:\n\nArrow Keys: Move\nF: Enter Vehicle".to_string();
        }
        GameState::Driving => {
            **text = "CONTROLS - Driving:\n\nArrow Keys: Drive\nSpace: Turbo (Supercar)\nF: Exit Vehicle".to_string();
        }
        GameState::Flying => {
            **text = "CONTROLS - Helicopter:\n\nArrow Keys: Move\nShift: Up, Ctrl: Down\nF: Exit Helicopter".to_string();
        }
        GameState::Jetting => {
            **text = "CONTROLS - F16 Fighter:\n\nArrow Keys: Fly\nQ/E: Up/Down\nSpace: Afterburner\nF: Exit F16".to_string();
        }
    }
}

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
            
            **text = waypoint_info;
        }
    }
}
