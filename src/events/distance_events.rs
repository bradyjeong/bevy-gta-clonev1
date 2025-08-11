//! Distance calculation coordination events
//! 
//! These events replace direct distance cache function calls with proper event-driven
//! communication, maintaining plugin boundaries while providing cached distance data.

use bevy::prelude::*;

/// Request ID for matching distance requests with responses (4 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct DistanceRequestId(pub u32);

impl DistanceRequestId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Request distance calculation between two entities (16 bytes)
/// Sent by: LOD systems, culling systems
/// Handled by: distance cache service
#[derive(Event, Debug, Clone, Copy)]
pub struct RequestDistance {
    pub id: DistanceRequestId,
    pub entity1: Entity,
    pub entity2: Entity,
}

impl RequestDistance {
    pub fn new(id: DistanceRequestId, entity1: Entity, entity2: Entity) -> Self {
        Self { id, entity1, entity2 }
    }
}

/// Request distance to reference point (typically player) (20 bytes)
/// Sent by: LOD systems, culling systems
/// Handled by: distance cache service
#[derive(Event, Debug, Clone, Copy)]
pub struct RequestDistanceToReference {
    pub id: DistanceRequestId,
    pub entity: Entity,
    pub reference_point: Vec3,
}

impl RequestDistanceToReference {
    pub fn new(id: DistanceRequestId, entity: Entity, reference_point: Vec3) -> Self {
        Self { id, entity, reference_point }
    }
}

/// Distance calculation result (24 bytes)
/// Sent by: distance cache service
/// Handled by: LOD systems, culling systems
#[derive(Event, Debug, Clone, Copy)]
pub struct DistanceResult {
    pub id: DistanceRequestId,
    pub entity1: Entity,
    pub entity2: Entity,
    pub distance: f32,
    pub distance_squared: f32,
    pub cached: bool,
}

impl DistanceResult {
    pub fn new(
        id: DistanceRequestId,
        entity1: Entity,
        entity2: Entity,
        distance: f32,
        cached: bool,
    ) -> Self {
        Self {
            id,
            entity1,
            entity2,
            distance,
            distance_squared: distance * distance,
            cached,
        }
    }
}

/// Distance to reference result (20 bytes)
/// Sent by: distance cache service
/// Handled by: LOD systems, culling systems
#[derive(Event, Debug, Clone, Copy)]
pub struct DistanceToReferenceResult {
    pub id: DistanceRequestId,
    pub entity: Entity,
    pub distance: f32,
    pub distance_squared: f32,
    pub cached: bool,
}

impl DistanceToReferenceResult {
    pub fn new(
        id: DistanceRequestId,
        entity: Entity,
        distance: f32,
        cached: bool,
    ) -> Self {
        Self {
            id,
            entity,
            distance,
            distance_squared: distance * distance,
            cached,
        }
    }
}

// Compile-time size verification (≤128 bytes requirement)
const _: () = {
    assert!(std::mem::size_of::<RequestDistance>() <= 128);
    assert!(std::mem::size_of::<RequestDistanceToReference>() <= 128);
    assert!(std::mem::size_of::<DistanceResult>() <= 128);
    assert!(std::mem::size_of::<DistanceToReferenceResult>() <= 128);
};
