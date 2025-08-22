use crate::components::{Player, VehicleBeacon, WaypointText};
use bevy::prelude::*;

/// Update waypoint system - shows distance to vehicles
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

/// Update beacon visibility system - uses Timer for square wave instead of sine
pub fn update_beacon_visibility(
    mut beacon_query: Query<&mut Visibility, With<VehicleBeacon>>,
    time: Res<Time>,
) {
    // Use integer division for clean square wave instead of sine > 0 which causes flashes
    let flash_cycle = ((time.elapsed_secs() * 2.0) as i32) % 2 == 0;

    for mut visibility in beacon_query.iter_mut() {
        *visibility = if flash_cycle {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
