//! Bridge system that converts validation results to spawn requests
//! 
//! This system completes the validation→spawn event flow by converting successful
//! validation results into spawn requests that can be processed by content handlers.

use bevy::prelude::*;
use crate::events::world::validation_events::SpawnValidationResult;
use crate::events::world::content_events::RequestDynamicSpawn;

/// Convert successful spawn validation results into dynamic spawn requests.
///
/// This system acts as the critical bridge in the event flow:
/// SpawnValidationResult (validation passed) → RequestDynamicSpawn (spawn entity)
///
/// Only emits spawn requests for validation results marked as valid,
/// ensuring only properly validated content gets spawned.
/// 
/// Named: handle_validation_to_spawn_bridge (per Oracle requirements)
pub fn handle_validation_to_spawn_bridge(
    mut validation_reader: EventReader<SpawnValidationResult>,
    mut spawn_writer: EventWriter<RequestDynamicSpawn>,
) {
    for result in validation_reader.read() {
        println!("🎯 SPAWN EVENT: Received SpawnValidationResult valid={} for {:?} at {:?}", 
            result.valid, result.content_type, result.position);
        
        if result.valid {
            spawn_writer.write(RequestDynamicSpawn::new(result.position, result.content_type));
            println!("🎯 SPAWN EVENT: Sent RequestDynamicSpawn for {:?} at {:?}", 
                result.content_type, result.position);
        }
    }
}


