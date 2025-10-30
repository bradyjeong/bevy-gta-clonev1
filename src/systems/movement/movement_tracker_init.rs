use bevy::prelude::*;
use crate::components::MovementTracker;

/// Initialize MovementTracker.last_position to current Transform position
/// Prevents first-frame "teleport spike" when using Default::default()
pub fn initialize_movement_tracker(
    mut query: Query<(&Transform, &mut MovementTracker), Added<MovementTracker>>,
) {
    for (transform, mut tracker) in query.iter_mut() {
        tracker.last_position = transform.translation;
    }
}
