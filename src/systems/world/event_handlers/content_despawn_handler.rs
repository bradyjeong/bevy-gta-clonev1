//! Dynamic content despawn handler using marker components
//! 
//! Processes entities marked for despawn instead of using events.
//! This is more efficient as it uses query-based processing.

use bevy::prelude::*;
use crate::components::dynamic_content::{DynamicContent, MarkedForDespawn};
#[allow(unused_imports)] // Will be removed after full migration
use crate::events::world::content_events::{RequestDynamicDespawn, DynamicContentDespawned};

/// Process entities marked for despawn
/// This replaces the event-based despawn system with a query-based approach
pub fn process_marked_for_despawn(
    mut commands: Commands,
    query: Query<Entity, (With<DynamicContent>, With<MarkedForDespawn>)>,
    #[cfg(feature = "legacy-events")]
    mut despawn_event_writer: EventWriter<DynamicContentDespawned>,
) {
    for entity in query.iter() {
        // Despawn the entity (removing DynamicContent triggers observer)
        commands.entity(entity).despawn();
        
        // Legacy event emission for compatibility
        #[cfg(feature = "legacy-events")]
        despawn_event_writer.write(DynamicContentDespawned::new(entity));
        
        trace!("Despawned entity {:?} marked for despawn", entity);
    }
}

/// Legacy handler for RequestDynamicDespawn events
/// Converts events to marker components for processing
#[cfg(feature = "legacy-events")]
pub fn handle_despawn_request_events(
    mut commands: Commands,
    mut despawn_reader: EventReader<RequestDynamicDespawn>,
) {
    for request in despawn_reader.read() {
        // Mark entity for despawn instead of immediate despawn
        if let Some(mut entity_commands) = commands.get_entity(request.entity) {
            entity_commands.insert(MarkedForDespawn);
            trace!("Marked entity {:?} for despawn from legacy event", request.entity);
        }
    }
}

/// Mark entities for despawn based on distance
/// This is an example of converting from event-based to component-based despawning
pub fn mark_distant_content_for_despawn(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &DynamicContent), Without<MarkedForDespawn>>,
    player_query: Query<&Transform, With<crate::components::player::Player>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    
    let player_pos = player_transform.translation;
    
    for (entity, transform, content) in query.iter() {
        let distance = player_pos.distance(transform.translation);
        
        if distance > content.content_type.despawn_distance() {
            commands.entity(entity).insert(MarkedForDespawn);
            trace!(
                "Marked {:?} for despawn - distance {} exceeds limit {}",
                content.content_type,
                distance,
                content.content_type.despawn_distance()
            );
        }
    }
}
