// DISABLED: Parallel physics systems removed due to conflicts with Rapier physics
// These systems were causing velocity explosions and physics instability

use bevy::prelude::*;
// Stub plugin to maintain compatibility
pub struct ParallelPhysicsPlugin;
impl Plugin for ParallelPhysicsPlugin {
    fn build(&self, _app: &mut App) {
        // No systems added - parallel physics disabled
    }
}
