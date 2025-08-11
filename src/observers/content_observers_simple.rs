//! Simplified content lifecycle tracking using query filters
//! 
//! This provides a bridge to more efficient entity tracking without
//! full observer pattern implementation.

use bevy::prelude::*;
use crate::components::DynamicContent;

/// Plugin that adds efficient content lifecycle tracking
pub struct ContentObserverPlugin;

impl Plugin for ContentObserverPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                PostUpdate,
                (
                    track_spawned_content,
                    track_removed_content,
                )
            );
    }
}

/// Track newly spawned content entities using Added filter
fn track_spawned_content(
    query: Query<(Entity, &Transform, &DynamicContent), Added<DynamicContent>>,
) {
    for (entity, transform, content) in query.iter() {
        trace!(
            "Content spawned: entity={:?}, type={:?}, pos={:?}",
            entity,
            content.content_type,
            transform.translation
        );
    }
}

/// Track removed content entities using RemovedComponents  
fn track_removed_content(
    mut removed: RemovedComponents<DynamicContent>,
) {
    for entity in removed.read() {
        trace!("Content removed: entity={:?}", entity);
    }
}
