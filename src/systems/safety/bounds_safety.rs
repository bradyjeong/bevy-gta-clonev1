use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::safety::{HighSpeed, WorldBounds};
use crate::components::ActiveEntity;

/// Generic bounds safety system for all high-speed entities
/// Replaces F-16 specific position_bounds.rs with NaN protection
pub fn bounds_safety_system(
    mut commands: Commands,
    bounds: Res<WorldBounds>,
    mut high_speed_query: Query<(
        Entity,
        &mut Transform, 
        &mut Velocity, 
        &HighSpeed
    ), (With<RigidBody>, With<ActiveEntity>)>,
) {
    for (entity, mut transform, mut velocity, _high_speed) in high_speed_query.iter_mut() {
        let pos = transform.translation;
        let max_coord = bounds.max_coordinate;
        
        // Critical: Check for NaN/Inf coordinates that cause Rapier panics
        if !pos.is_finite() || !velocity.linvel.is_finite() || !velocity.angvel.is_finite() {
            error!("Entity {:?} has invalid coordinates/velocity - despawning to prevent Rapier panic", entity);
            commands.entity(entity).despawn();
            continue;
        }
        
        // Check if any coordinate exceeds bounds
        if pos.x.abs() > max_coord || pos.y.abs() > max_coord || pos.z.abs() > max_coord {
            warn!("High-speed entity {:?} at {:?} exceeded world bounds, performing emergency reset", 
                  entity, pos);
            
            // Emergency reset
            let safe_position = Vec3::new(
                pos.x.clamp(-max_coord * 0.8, max_coord * 0.8),
                pos.y.max(1.0).min(max_coord * 0.8),
                pos.z.clamp(-max_coord * 0.8, max_coord * 0.8),
            );
            
            transform.translation = safe_position;
            velocity.linvel *= bounds.emergency_damping;
            velocity.angvel *= bounds.emergency_damping;
            
            info!("Emergency reset: entity {:?} moved from {:?} to {:?}", 
                  entity, pos, safe_position);
        }
    }
}

/// Diagnostic system for bounds monitoring
pub fn bounds_diagnostics_system(
    time: Res<Time>,
    bounds: Res<WorldBounds>,
    high_speed_query: Query<(Entity, &Transform), (With<HighSpeed>, With<ActiveEntity>)>,
) {
    // Only log every 5 seconds to avoid spam
    if (time.elapsed_secs() % 5.0) < time.delta_secs() {
        for (entity, transform) in high_speed_query.iter() {
            let pos = transform.translation;
            let distance_from_origin = pos.length();
            
            if distance_from_origin > bounds.max_coordinate * 0.5 {
                info!("High-speed entity {:?} position check: {:?} (distance: {:.0}m from origin)", 
                      entity, pos, distance_from_origin);
            }
        }
    }
}
