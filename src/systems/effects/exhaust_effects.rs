use bevy::prelude::*;
use crate::components::ExhaustFlame;

/// Exhaust effects system - cleans up old exhaust flames
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
