//! Test system to verify event-driven spawning architecture works
//! This system sends a test spawn validation request to trigger the full chain:
//! RequestSpawnValidation → SpawnValidationResult → RequestDynamicSpawn → entity spawn

use bevy::prelude::*;
use crate::events::world::validation_events::{RequestSpawnValidation, ValidationId};
use crate::components::world::ContentType;

/// Test system that sends a spawn validation request after 3 seconds
/// This tests the complete event-driven spawning chain from architectural_shift.md
pub fn test_spawn_validation_chain(
    mut commands: Commands,
    mut validation_writer: EventWriter<RequestSpawnValidation>,
    time: Res<Time>,
    mut test_sent: Local<bool>,
) {
    // Send test request after 3 seconds, only once
    if !*test_sent && time.elapsed_secs() > 3.0 {
        *test_sent = true;
        
        info!("🧪 TEST: Sending RequestSpawnValidation for test vehicle");
        
        validation_writer.send(RequestSpawnValidation::new(
            ValidationId::new(999999), // Unique test ID
            Vec3::new(12.0, 1.0, 5.0), // Position near player, away from test objects
            ContentType::Vehicle,
        ));
    }
}
