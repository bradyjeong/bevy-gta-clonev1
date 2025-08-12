use bevy::prelude::*;

/// Event fired when the active player enters a new chunk
#[derive(Event, Clone, Copy, Debug)]
pub struct PlayerChunkChanged {
    pub entity: Entity,
    pub new_chunk: (i32, i32),
    pub old_chunk: Option<(i32, i32)>,
    pub position: Vec3,
}

impl PlayerChunkChanged {
    pub fn new(entity: Entity, new_chunk: (i32, i32), old_chunk: Option<(i32, i32)>, position: Vec3) -> Self {
        Self {
            entity,
            new_chunk,
            old_chunk,
            position,
        }
    }
}
