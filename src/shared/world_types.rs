use bevy::prelude::*;

/// Shared culling component to break circular dependencies
#[derive(Component, Default, Debug, Clone)]
pub struct SharedCullable {
    pub cull_distance: f32,
    pub last_distance: f32,
    pub is_visible: bool,
    pub priority: u8,
}

impl SharedCullable {
    pub fn new(cull_distance: f32, priority: u8) -> Self {
        Self {
            cull_distance,
            last_distance: f32::MAX,
            is_visible: true,
            priority,
        }
    }
}


