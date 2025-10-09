use crate::components::{ActiveEntity, HumanAnimation, HumanMovement, NPC};

use bevy::prelude::*;
use bevy::render::view::visibility::VisibilityRange;
use bevy_rapier3d::prelude::*;
use rand::Rng;

/// Simple NPC movement that follows direct AI patterns
#[allow(clippy::type_complexity)]
pub fn simple_npc_movement(
    time: Res<Time>,
    mut npc_query: Query<
        (
            Entity,
            &mut Transform,
            &mut Velocity,
            &mut NPC,
            &mut HumanMovement,
            &mut HumanAnimation,
        ),
        With<VisibilityRange>,
    >,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<NPC>)>,
) {
    let current_time = time.elapsed_secs();

    // Get player position for distance-based optimization
    let player_pos = if let Ok(active_transform) = active_query.single() {
        active_transform.translation
    } else {
        Vec3::ZERO
    };

    for (_entity, mut transform, mut velocity, mut npc, mut movement, mut animation) in
        npc_query.iter_mut()
    {
        // Note: With VisibilityRange, Bevy handles culling automatically
        // NPCs continue to update their AI even when not visible

        // Only update NPCs at their specific intervals (staggered updates)
        if current_time - npc.last_update < npc.update_interval {
            continue;
        }
        npc.last_update = current_time;

        let current_pos = transform.translation;
        let target_pos = npc.target_position;

        // Calculate distance to target (XZ plane only to avoid vertical interference)
        let distance = (current_pos.xz() - target_pos.xz()).length();

        // Reduce update frequency for distant NPCs
        let distance_to_player = current_pos.distance(player_pos);
        if distance_to_player > 100.0 {
            npc.update_interval = 0.5; // Very slow updates for distant NPCs
        } else if distance_to_player > 50.0 {
            npc.update_interval = 0.2; // Slower updates for far NPCs
        } else {
            npc.update_interval = 0.05; // Normal updates for close NPCs
        }

        // If close to target, pick a new random target (preserve current Y)
        if distance < 5.0 {
            npc.target_position = Vec3::new(
                rand::thread_rng().gen_range(-900.0..900.0),
                current_pos.y,
                rand::thread_rng().gen_range(-900.0..900.0),
            );
        } else {
            // Simple, direct NPC movement (XZ plane only)
            let flat = (target_pos - current_pos).with_y(0.0);
            if flat.length_squared() > 1e-4 {
                let dir = flat.normalize();
                velocity.linvel.x = dir.x * npc.speed;
                velocity.linvel.z = dir.z * npc.speed;
                // Keep velocity.linvel.y unchanged (preserve gravity)

                // Update movement tracking for animation
                movement.current_speed = Vec2::new(velocity.linvel.x, velocity.linvel.z).length();
                movement.target_velocity = Vec3::new(dir.x * npc.speed, 0.0, dir.z * npc.speed);

                // Update animation state
                animation.is_walking = movement.current_speed > 0.3;
                animation.is_running = false;

                // Face movement direction
                transform.rotation = Quat::from_rotation_y(dir.x.atan2(dir.z));
            }
        }
    }
}

/// Legacy NPC movement system - kept for backwards compatibility
pub fn optimized_npc_movement(
    time: Res<Time>,
    mut npc_query: Query<(&mut Transform, &mut Velocity, &mut NPC), With<VisibilityRange>>,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<NPC>)>,
) {
    let current_time = time.elapsed_secs();

    // Get player position for distance-based optimization
    let player_pos = if let Ok(active_transform) = active_query.single() {
        active_transform.translation
    } else {
        Vec3::ZERO
    };

    for (mut transform, mut velocity, mut npc) in npc_query.iter_mut() {
        // Note: With VisibilityRange, Bevy handles culling automatically

        // Only update NPCs at their specific intervals (staggered updates)
        if current_time - npc.last_update < npc.update_interval {
            continue;
        }
        npc.last_update = current_time;

        let current_pos = transform.translation;
        let target_pos = npc.target_position;

        // Calculate distance to target
        let distance = current_pos.distance(target_pos);

        // Reduce update frequency for distant NPCs
        let distance_to_player = current_pos.distance(player_pos);
        if distance_to_player > 100.0 {
            npc.update_interval = 0.5; // Very slow updates for distant NPCs
        } else if distance_to_player > 50.0 {
            npc.update_interval = 0.2; // Slower updates for far NPCs
        } else {
            npc.update_interval = 0.05; // Normal updates for close NPCs
        }

        // If close to target, pick a new random target
        if distance < 5.0 {
            npc.target_position = Vec3::new(
                rand::thread_rng().gen_range(-900.0..900.0),
                1.0,
                rand::thread_rng().gen_range(-900.0..900.0),
            );
        } else {
            // Move towards target (legacy implementation)
            let direction = (target_pos - current_pos).normalize();
            velocity.linvel = Vec3::new(
                direction.x * npc.speed,
                velocity.linvel.y, // Preserve gravity
                direction.z * npc.speed,
            );

            // Face movement direction
            if direction.length() > 0.1 {
                let rotation = Quat::from_rotation_y((-direction.x).atan2(-direction.z));
                transform.rotation = rotation;
            }
        }
    }
}
