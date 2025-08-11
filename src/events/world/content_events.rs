//! Dynamic content spawning coordination events
//! 
//! These events decouple dynamic content spawning from world generation,
//! enabling flexible content placement without direct system coupling.

use bevy::prelude::*;

/// Types of dynamic content that can be spawned (1 byte)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ContentType {
    Road = 0,
    Building = 1,
    Tree = 2,
    Vehicle = 3,
    NPC = 4,
}

/// Request to spawn dynamic content at a specific location (16 bytes)
/// 
/// This event represents a validated request to create new dynamic content.
/// It's emitted after position validation has passed and guarantees the
/// location is suitable for the requested content type.
/// 
/// # Event Flow
/// ValidationToSpawnBridge → RequestDynamicSpawn → ContentSpawnHandler
/// 
/// # Performance Notes
/// - Small event size (16 bytes) for efficient event processing
/// - Copy-able to avoid allocation overhead
/// - Processed within same frame as validation (0-frame latency)
/// 
/// Sent by: validation→spawn bridge system, mission systems
/// Handled by: content spawning system
#[derive(Event, Debug, Clone, Copy)]
pub struct RequestDynamicSpawn {
    pub pos: Vec3,
    pub kind: ContentType,
}

impl RequestDynamicSpawn {
    pub fn new(pos: Vec3, kind: ContentType) -> Self {
        Self { pos, kind }
    }
    
    pub fn vehicle(pos: Vec3) -> Self {
        Self::new(pos, ContentType::Vehicle)
    }
    
    pub fn building(pos: Vec3) -> Self {
        Self::new(pos, ContentType::Building)
    }
    
    pub fn npc(pos: Vec3) -> Self {
        Self::new(pos, ContentType::NPC)
    }
}

/// Notification that dynamic content has been successfully spawned (20 bytes)
/// 
/// This event confirms successful entity creation and provides the entity ID
/// for systems that need to track or reference the spawned content.
/// 
/// # Event Flow  
/// ContentSpawnHandler → DynamicContentSpawned → TrackingSystems/UI/Missions
/// 
/// # Use Cases
/// - Update entity count tracking and performance metrics
/// - Notify UI systems of new content for debugging overlays
/// - Enable mission systems to reference spawned entities
/// - Update spatial indexing structures for culling
/// 
/// # Performance Notes
/// - Emitted once per spawned entity to minimize event volume
/// - Contains minimal data for fast processing
/// - Entity references are safe and guaranteed valid when event is sent
/// 
/// Sent by: content spawning system
/// Handled by: tracking systems, UI systems, mission systems
#[derive(Event, Debug, Clone, Copy)]
pub struct DynamicContentSpawned {
    pub entity: Entity,
    pub pos: Vec3,
    pub kind: ContentType,
}

impl DynamicContentSpawned {
    pub fn new(entity: Entity, pos: Vec3, kind: ContentType) -> Self {
        Self { entity, pos, kind }
    }
}

/// Request to despawn dynamic content (8 bytes)
/// Sent by: culling system, cleanup systems
/// Handled by: content despawning system
#[derive(Event, Debug, Clone, Copy)]
pub struct RequestDynamicDespawn {
    pub entity: Entity,
}

impl RequestDynamicDespawn {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}

/// Notification that dynamic content has been despawned (8 bytes)
/// Sent by: content despawning system
/// Handled by: tracking systems, UI systems
#[derive(Event, Debug, Clone, Copy)]
pub struct DynamicContentDespawned {
    pub entity: Entity,
}

impl DynamicContentDespawned {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}

// Compile-time size verification (≤128 bytes requirement)
const _: () = {
    assert!(std::mem::size_of::<RequestDynamicSpawn>() <= 128);
    assert!(std::mem::size_of::<DynamicContentSpawned>() <= 128);
    assert!(std::mem::size_of::<RequestDynamicDespawn>() <= 128);
    assert!(std::mem::size_of::<DynamicContentDespawned>() <= 128);
};
