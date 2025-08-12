use bevy::prelude::*;
use crate::components::ActiveEntity;
use crate::events::PlayerChunkChanged;

/// Resource to track the active player's current chunk
#[derive(Resource, Default)]
pub struct PlayerChunkTracker {
    pub current_chunk: Option<(i32, i32)>,
    pub chunk_size: f32,
}

impl PlayerChunkTracker {
    pub fn new(chunk_size: f32) -> Self {
        Self {
            current_chunk: None,
            chunk_size,
        }
    }
    
    /// Calculate chunk coordinates from world position
    pub fn world_to_chunk(&self, position: Vec3) -> (i32, i32) {
        (
            (position.x / self.chunk_size).floor() as i32,
            (position.z / self.chunk_size).floor() as i32,
        )
    }
}

/// System that tracks active player movement and fires PlayerChunkChanged events
pub fn track_player_chunk_changes(
    mut chunk_tracker: ResMut<PlayerChunkTracker>,
    mut chunk_changed_events: EventWriter<PlayerChunkChanged>,
    active_query: Query<(Entity, &Transform), (With<ActiveEntity>, Changed<Transform>)>,
) {
    for (entity, transform) in active_query.iter() {
        let current_chunk = chunk_tracker.world_to_chunk(transform.translation);
        
        // Check if chunk has changed
        if chunk_tracker.current_chunk != Some(current_chunk) {
            let old_chunk = chunk_tracker.current_chunk;
            chunk_tracker.current_chunk = Some(current_chunk);
            
            // Fire event for chunk change
            chunk_changed_events.write(PlayerChunkChanged::new(
                entity,
                current_chunk,
                old_chunk,
                transform.translation,
            ));
        }
    }
}
