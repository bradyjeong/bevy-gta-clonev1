use bevy::prelude::*;

/// Marker component for any object that exists in the world
#[derive(Component, Debug, Default)]
#[component(immutable)]  // Object type and radius are static after spawn
pub struct WorldObject {
    /// Type of world object
    pub object_type: WorldObjectType,
    /// Radius for collision detection
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorldObjectType {
    #[default]
    Generic,
    Building,
    Vehicle,
    Vegetation,
    Prop,
    Road,
}

impl WorldObject {
    pub fn new(object_type: WorldObjectType, radius: f32) -> Self {
        Self {
            object_type,
            radius,
        }
    }
}
