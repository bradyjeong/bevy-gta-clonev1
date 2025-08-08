use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{F16, ActiveEntity, AircraftFlight};

/// System to prevent entities from escaping world bounds and causing physics crashes
/// This is critical for high-speed entities like the F16 fighter jet
pub fn world_bounds_safety_system(
    mut f16_query: Query<(&mut Transform, &mut Velocity, &mut AircraftFlight), (With<F16>, With<ActiveEntity>)>,
) {
    // Rapier's safe coordinate range - beyond this causes crashes
    const MAX_SAFE_COORDINATE: f32 = 50000.0;
    const RESET_BOUNDARY: f32 = MAX_SAFE_COORDINATE * 0.9; // Reset before hitting the limit
    
    for (mut transform, mut velocity, mut flight) in f16_query.iter_mut() {
        let pos = transform.translation;
        
        // Check if any coordinate exceeds safe bounds
        if pos.x.abs() > RESET_BOUNDARY || 
           pos.y.abs() > RESET_BOUNDARY || 
           pos.z.abs() > RESET_BOUNDARY {
            
            warn!("F16 approaching world bounds at {:?}, performing emergency reset", pos);
            
            // Calculate safe return position
            let safe_position = Vec3::new(
                pos.x.clamp(-RESET_BOUNDARY * 0.8, RESET_BOUNDARY * 0.8),
                pos.y.clamp(50.0, 5000.0), // Reasonable flight altitude
                pos.z.clamp(-RESET_BOUNDARY * 0.8, RESET_BOUNDARY * 0.8)
            );
            
            // Emergency reset
            transform.translation = safe_position;
            
            // Reduce velocity to prevent immediate re-escape
            velocity.linvel *= 0.2;
            velocity.angvel *= 0.2;
            
            // Reset flight controls to prevent continued acceleration
            flight.throttle *= 0.5;
            flight.afterburner_active = false;
            flight.afterburner = false;
            
            info!("F16 emergency reset: moved from {:?} to {:?}", pos, safe_position);
        }
    }
}

/// System to monitor and report entity positions for debugging
pub fn position_monitor_system(
    f16_query: Query<&Transform, (With<F16>, With<ActiveEntity>)>,
    time: Res<Time>,
) {
    // Only log occasionally to avoid spam
    if time.elapsed_secs() % 5.0 < 0.1 {
        for transform in f16_query.iter() {
            let pos = transform.translation;
            let distance_from_origin = pos.length();
            
            if distance_from_origin > 10000.0 {
                info!("F16 position check: {:?} (distance: {:.0}m from origin)", pos, distance_from_origin);
            }
        }
    }
}
