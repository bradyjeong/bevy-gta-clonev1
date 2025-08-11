//! Dynamic content component definitions
//! 
//! Components for entities that are dynamically spawned and despawned
//! during gameplay based on distance, chunk loading, or other criteria.

use bevy::prelude::*;

/// Component marking an entity as dynamic content that can be spawned/despawned
#[derive(Component, Debug, Clone)]
pub struct DynamicContent {
    pub content_type: ContentType,
}

impl DynamicContent {
    pub fn new(content_type: ContentType) -> Self {
        Self { content_type }
    }
}

/// Marker component for entities that should be despawned
/// This replaces the RequestDynamicDespawn event
#[derive(Component)]
pub struct MarkedForDespawn;

/// Types of dynamic content in the game world
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Road,
    Building,
    Tree,
    Vehicle,
    NPC,
}

impl ContentType {
    /// Get display name for debug purposes
    pub fn display_name(&self) -> &'static str {
        match self {
            ContentType::Road => "Road",
            ContentType::Building => "Building",
            ContentType::Tree => "Tree",
            ContentType::Vehicle => "Vehicle",
            ContentType::NPC => "NPC",
        }
    }
    
    /// Get expected despawn distance for this content type
    pub fn despawn_distance(&self) -> f32 {
        match self {
            ContentType::Road => 500.0,
            ContentType::Building => 300.0,
            ContentType::Tree => 200.0,
            ContentType::Vehicle => 150.0,
            ContentType::NPC => 100.0,
        }
    }
}

/// Conversion from world ContentType to dynamic_content ContentType
impl From<crate::components::world::ContentType> for ContentType {
    fn from(world_type: crate::components::world::ContentType) -> Self {
        match world_type {
            crate::components::world::ContentType::Road => ContentType::Road,
            crate::components::world::ContentType::Building => ContentType::Building,
            crate::components::world::ContentType::Tree => ContentType::Tree,
            crate::components::world::ContentType::Vehicle => ContentType::Vehicle,
            crate::components::world::ContentType::NPC => ContentType::NPC,
        }
    }
}
