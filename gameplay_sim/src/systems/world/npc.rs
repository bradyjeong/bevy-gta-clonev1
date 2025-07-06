use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use std::cell::RefCell;
use game_core::components::{NPC, Cullable, ActiveEntity};
use crate::systems::input::ControlManager;

thread_local! {
    static NPC_RNG: RefCell<rand::rngs::ThreadRng> = RefCell::new(rand::thread_rng());
}

/// Updated NPC movement that uses the unified control system
pub fn unified_npc_movement(
    time: Res<Time>,
    control_manager: Res<ControlManager>,
    mut npc_query: Query<(Entity, &mut Transform, &mut Velocity, &mut NPC, &Cullable)>,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<NPC>)>,
) {
    let current_time = time.elapsed_secs();
    
    // Get player position for distance-based optimization
    let player_pos = if let Ok(active_transform) = active_query.single() {
        active_transform.translation
    } else {
        Vec3::ZERO
    };
    
    for (entity, mut transform, mut velocity, mut npc, cullable) in npc_query.iter_mut() {
        // Skip if culled
        if cullable.is_culled {
            velocity.linvel = Vec3::ZERO;
            continue;
        }
        
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
                NPC_RNG.with(|rng| rng.borrow_mut().gen_range(-900.0..900.0)),
                1.0,
                NPC_RNG.with(|rng| rng.borrow_mut().gen_range(-900.0..900.0)),
            );
        } else {
            // Use unified control system for NPC movement
            if let Some(ai_decision) = control_manager.get_ai_decision(entity) {
                let direction = ai_decision.movement_direction;
                if direction.length() > 0.1 {
                    velocity.linvel = Vec3::new(
                        direction.x * npc.speed * ai_decision.speed_factor,
                        velocity.linvel.y, // Preserve gravity
                        direction.z * npc.speed * ai_decision.speed_factor,
                    );
                    
                    // Face movement direction with AI decision rotation
                    let rotation_input = ai_decision.rotation_target;
                    if rotation_input.abs() > 0.1 {
                        let rotation = Quat::from_rotation_y((-direction.x).atan2(-direction.z) + rotation_input * 0.1);
                        transform.rotation = rotation;
                    }
                }
            } else {
                // Fallback to original movement if no AI decision
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
}

/// Legacy NPC movement system - kept for backwards compatibility
pub fn optimized_npc_movement(
    time: Res<Time>,
    mut npc_query: Query<(&mut Transform, &mut Velocity, &mut NPC, &Cullable)>,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<NPC>)>,
) {
    let current_time = time.elapsed_secs();
    
    // Get player position for distance-based optimization
    let player_pos = if let Ok(active_transform) = active_query.single() {
        active_transform.translation
    } else {
        Vec3::ZERO
    };
    
    for (mut transform, mut velocity, mut npc, cullable) in npc_query.iter_mut() {
        // Skip if culled
        if cullable.is_culled {
            velocity.linvel = Vec3::ZERO;
            continue;
        }
        
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
                NPC_RNG.with(|rng| rng.borrow_mut().gen_range(-900.0..900.0)),
                1.0,
                NPC_RNG.with(|rng| rng.borrow_mut().gen_range(-900.0..900.0)),
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
