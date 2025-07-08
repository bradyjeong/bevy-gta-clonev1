use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use game_core::prelude::{Player, ActiveEntity};

/// Physics-safe collision resolution using Rapier authority
pub fn player_collision_resolution_system(
    mut player_query: Query<(Entity, &mut Velocity, &Transform), (With<Player>, With<ActiveEntity>)>,
) {
    let Ok((_player_entity, mut velocity, transform)) = player_query.single_mut() else {
        return;
    };

    let player_position = transform.translation;
    
    // Only handle extreme position cases using velocity-based approach
    if player_position.y < -10.0 {
        warn!("Player far below ground at y={:.2}, applying upward force", player_position.y);
        // Apply strong upward force to lift player to safe position
        velocity.linvel = Vec3::new(0.0, 15.0, 0.0);
        velocity.angvel = Vec3::ZERO;
    }
    
    // Only handle extreme world boundaries
    let max_coord = 2000.0;
    if player_position.x.abs() > max_coord || player_position.z.abs() > max_coord {
        warn!("Player at extreme position {:?}, applying corrective force", player_position);
        // Apply force toward center of world
        let correction_direction = -player_position.normalize_or_zero();
        velocity.linvel = correction_direction * 10.0;
        velocity.angvel = Vec3::ZERO;
    }
}

/// Physics-safe ground collision using velocity modification
pub fn player_movement_validation_system(
    mut player_query: Query<(&mut Velocity, &Transform), (With<Player>, With<ActiveEntity>)>,
) {
    let Ok((mut velocity, transform)) = player_query.single_mut() else {
        return;
    };

    // Use velocity-based ground collision instead of direct position modification
    if transform.translation.y < -1.0 {
        // Stop downward movement and add upward force
        if velocity.linvel.y < 0.0 {
            velocity.linvel.y = 0.0;
        }
        velocity.linvel.y += 5.0; // Upward force to lift player above ground
    }
}
